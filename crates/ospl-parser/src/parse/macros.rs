use crate::{lexer::token::{Span, Token, exp_ident}, parse::{Parser, Res, ffi::exp_addr}, tComb, tExp};

#[derive(Debug, Eq, Hash, Clone, PartialEq)]
pub struct MacroIdent(pub String);

#[derive(Debug, Clone, PartialEq)]
pub enum MacroExpr {
    /* list creation */
    NewList(Vec<MacroExpr>),

    /* parse functions */
    Next(MacroTokType),

    /// Matches any of the given tokens
    NextToken(Vec<Token>),

    Address(u64),

    Do(Vec<MacroStmt>),
    Template(Vec<MacroExpr>),

    Var(MacroIdent),
    LiteralToken(Span),
    BinaryAdd(Box<MacroExpr>, Box<MacroExpr>),
    ListGet(Box<MacroExpr>, Box<MacroExpr>),
}

#[derive(Debug, Clone, PartialEq)]
pub enum MacroStmt {
    Define(MacroIdent, MacroExpr),
    Assign(MacroIdent, MacroExpr),
    Yeild(MacroExpr),
    Loop(Vec<MacroStmt>),
    For(MacroIdent, MacroExpr, Vec<MacroStmt>),
}

#[derive(Debug, Clone, PartialEq)]
pub enum MacroTokType { Expr, Atom, Ident, Stmt, Type }

#[derive(Debug, Clone, PartialEq)]
pub struct Macro {
    pub body: Vec<MacroStmt>,
    pub return_type: MacroTokType
}

impl<'a> Parser<'a> {
    pub fn parse_ident(&mut self) -> Res<String> {
        let (_, Token::Ident(name)) = self.expect(exp_ident())?.destructure()
        else { unreachable!() };

        return Ok(name)
    }

    fn parse_macro_expr_inner(&mut self) -> Res<MacroExpr> {
        let s = self.expect(tComb!(
            "LBracket | DollarSign | Ident | ListT | Continue | Do | AddressLiteral",
            tExp!(LBracket, ListT, Continue, DollarSign, Do),
            exp_addr(),
            exp_ident(),
        ))?;

        match s.token() {
            Token::ListT => {
                let mut items = Vec::new();
                self.expect(tExp!(LSquirly))?;
                loop {
                    if *self.peek()?.token() == Token::RSquirly {
                        self.next()?;
                        break;
                    }

                    items.push(self.parse_macro_expr()?);
                }

                return Ok(MacroExpr::NewList(items));
            },
            Token::LBracket => {
                let mut template = Vec::new();
                loop {
                    if *self.peek()?.token() == Token::RBracket
                    {
                        self.next()?;
                        break;
                    }

                    let x = self.parse_macro_expr()?;
                    template.push(x);
                }

                return Ok(MacroExpr::Template(template))
            }
            Token::Continue => {
                let t = self.parse_macro_type()?;
                return Ok(MacroExpr::Next(t))
            },
            Token::Ident(i) => {
                return Ok(MacroExpr::Var(MacroIdent(i.to_owned())));
            },
            Token::DollarSign => {
                let s1 = self.parse_macro_expr()?;
                let sop = self.expect(tExp!(Plus, Dot))?;

                match sop.token() {
                    Token::Plus => {
                        let s2 = self.parse_macro_expr()?;
                        return Ok(MacroExpr::BinaryAdd(Box::new(s1), Box::new(s2)));
                    },
                    Token::Dot => {
                        let s2 = self.parse_macro_expr()?;
                        return Ok(MacroExpr::ListGet(Box::new(s1), Box::new(s2)))
                    },
                    _ => unreachable!()
                }
            },
            Token::AddressLiteral(l) => {
                return Ok(MacroExpr::Address(*l))
            },
            Token::Do => {
                let blk = self.parse_macro_block()?;
                return Ok(MacroExpr::Do(blk))
            },
            _ => unimplemented!()
        }
    }

    pub fn parse_macro_expr(&mut self) -> Res<MacroExpr> {
        let s = self.next()?;
        match s.token() {
            Token::DollarSign => {
                return self.parse_macro_expr_inner();
            },
            _ => return Ok(MacroExpr::LiteralToken(s))
        }
    }

    pub fn parse_macro_stmt(&mut self) -> Res<MacroStmt> {
        let s = self.expect(tComb!(
            "Def | Return | Loop | For | Ident",
            tExp!(Def, Return, Loop, For),
            exp_ident(),
        ))?;

        match s.token() {
            Token::Def => {
                let name = self.parse_ident()?;
                self.expect(tExp!(Equals))?;
                let expr = self.parse_macro_expr()?;
                return Ok(MacroStmt::Define(MacroIdent(name), expr))
            },
            Token::Ident(i) => {
                self.expect(tExp!(Equals))?;
                let e = self.parse_macro_expr()?;
                return Ok(MacroStmt::Assign(MacroIdent(i.to_owned()), e))
            }
            Token::Return => {
                let expr = self.parse_macro_expr()?;
                return Ok(MacroStmt::Yeild(expr));
            },
            Token::Loop => {
                let blk = self.parse_macro_block()?;
                return Ok(MacroStmt::Loop(blk));
            },
            Token::For => {
                let i = MacroIdent(self.parse_ident()?);
                self.expect(tExp!(Equals))?;

                let e = self.parse_macro_expr()?;
                let blk = self.parse_macro_block()?;

                return Ok(MacroStmt::For(i, e, blk))
            }
            _ => unreachable!()
        }
    }

    pub fn parse_macro_block(&mut self) -> Res<Vec<MacroStmt>> {
        self.expect(tExp!(LSquirly))?;
        let mut contents = Vec::new();
        loop {
            if *self.peek()?.token() == Token::RSquirly {
                self.next()?;
                break;
            }

            contents.push(self.parse_macro_stmt()?);
            self.expect(tExp!(Semicolon))?;
        }

        return Ok(contents)
    }

    pub fn parse_macro_definition(&mut self) -> Res<(String, Macro)> {
        self.expect(tExp!(Macro))?;

        let name = self.parse_ident()?;

        self.expect(tExp!(Arrow))?;

        let return_type = self.parse_macro_type()?;

        /* main macro parsing logic! */
        let body = self.parse_macro_block()?;        

        return Ok((name, Macro {
            body,
            return_type
        }))
    }

    pub fn parse_macro_type(&mut self) -> Res<MacroTokType> {
        let (_, Token::Ident(name)) = self.expect(exp_ident())?.destructure()
        else { unreachable!() };

        return Ok(match name.as_ref() {
            "expr" => MacroTokType::Expr,
            "atom" => MacroTokType::Atom,
            "ident" => MacroTokType::Ident,
            "stmt" => MacroTokType::Stmt,
            "type" => MacroTokType::Type,
            _ => panic!("TODO error")
        })
    }
}
