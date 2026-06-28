use crate::{lexer::token::{exp_ident, Token, TokenExpectation}, parse::{PE, Parser, Res}, tExp};

/// A macro construct expands to tokens
#[derive(Clone, PartialEq, Debug)]
pub enum MacroConstructKind {
    Expr,
    Atom,
    Stmt,
    Ident,
    Type,
    SingleToken,
    SpecificTokens(Vec<Token>),
}

#[derive(Clone, PartialEq, Debug)]
pub enum MacroExpr {
    Parse(MacroConstructKind),
    Peek(MacroConstructKind),
    Var(String),
    Template(Vec<MacroExpr>),
    Literal(Token),
}

#[derive(Clone, PartialEq, Debug)]
pub enum MacroStatement {
    Define(String, MacroExpr),
    Emit(MacroExpr),
    Loop(Vec<MacroStatement>),
    If(MacroExpr, MacroExpr, Vec<MacroStatement>),
    IfNot(MacroExpr, MacroExpr, Vec<MacroStatement>),
    Do(MacroExpr),
    BreakLoop,
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub enum MacroControl { Default, BreakLoop }

#[derive(Clone, PartialEq, Debug)]
pub struct Macro {
    pub body: Vec<MacroStatement>,
}

impl<'a> Parser<'a> {
    pub fn parse_macro_construct_kind(&mut self) -> Res<MacroConstructKind> {
        let span = self.expect(TokenExpectation {
            label: "Ident(Expr) | Ident(Stmt)",
            matches: Box::new(|t| matches!(t, Token::Ident(_)))
        })?;

        return Ok(match span.token() {
            Token::Ident(s) => {
                let base = match s.as_str() {
                    "expr" => MacroConstructKind::Expr,
                    "stmt" => MacroConstructKind::Stmt,
                    "ident" => MacroConstructKind::Ident,
                    "type" => MacroConstructKind::Type,
                    "tok" => MacroConstructKind::SingleToken,
                    _ => return Err(PE::NoSuchMacroConstruct(s.to_owned()))
                };

                base
            }
            _ => unreachable!()
        });
    }

    pub fn parse_macro_expr(&mut self) -> Res<MacroExpr> {
        let span = self.next()?;

        match span.token() {
            Token::DollarSign => {
                let spn = self.next()?;
                match spn.token() {
                    Token::Continue => {
                        let construct = self.parse_macro_construct_kind()?;
                        return Ok(MacroExpr::Parse(construct))
                    },
                    Token::Try => {
                        let construct = self.parse_macro_construct_kind()?;
                        return Ok(MacroExpr::Peek(construct))
                    }
                    Token::LBracket => {
                        let mut template = Vec::new();
                        loop {
                            if *self.peek()?.token() == Token::RBracket {
                                self.next()?;
                                break;
                            }

                            template.push(self.parse_macro_expr()?);
                        };

                        return Ok(MacroExpr::Template(template))
                    },
                    Token::Ident(i) => {
                        return Ok(MacroExpr::Var(i.to_owned()))
                    }
                    _ => panic!()
                }
            },
            literal => Ok(MacroExpr::Literal(literal.to_owned())),
        }
    }

    pub fn parse_macro_block(&mut self) -> Res<Vec<MacroStatement>> {
        let mut body = Vec::new();
        self.expect(tExp!(LSquirly))?;
        loop {
            if *self.peek()?.token() == Token::RSquirly {
                self.next()?;
                break;
            }

            let stmt = self.parse_macro_stmt()?;
            body.push(stmt);
            self.expect(tExp!(Semicolon))?;
        }

        return Ok(body)
    }

    pub fn parse_macro_stmt(&mut self) -> Res<MacroStatement> {
        let stmt_starter = self.expect(tExp!(
            Def,
            Return,
            Loop,
            If,
            Break,
            Do,
        ))?;

        let stmt = match stmt_starter.token() {
            Token::Def => {
                let (_, Token::Ident(name)) = self.expect(exp_ident())?.destructure()
                else { unreachable!() };

                self.expect(tExp!(Equals))?;

                let e = self.parse_macro_expr()?;

                MacroStatement::Define(name, e)
            },
            Token::Return => {
                let e = self.parse_macro_expr()?;
                MacroStatement::Emit(e)
            },
            Token::Loop => {
                let body = self.parse_macro_block()?;
                MacroStatement::Loop(body)
            },
            Token::If => {
                let span = self.expect(tExp!(IsEqual, IsNotEqual))?;

                let one = self.parse_macro_expr()?;
                let two = self.parse_macro_expr()?;
                let body = self.parse_macro_block()?;

                match span.token() {
                    Token::IsEqual => {
                        MacroStatement::If(one, two, body)
                    },
                    Token::IsNotEqual => {
                        MacroStatement::IfNot(one, two, body)
                    },
                    _ => unreachable!()
                }
            },
            Token::Break => {
                MacroStatement::BreakLoop
            }
            Token::Do => {
                let expr = self.parse_macro_expr()?;
                MacroStatement::Do(expr)
            }
            _ => unreachable!()
        };

        return Ok(stmt)
    }

    pub fn parse_macro_definition(&mut self) -> Res<(String, Macro)> {
        let (_, Token::Ident(name)) = self.expect(exp_ident())?.destructure()
        else { unreachable!() };

        let body = self.parse_macro_block()?;

        return Ok((name, Macro {
            body,
        }))
    }
}
