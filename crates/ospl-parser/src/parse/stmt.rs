use ospl_common::ast::{Statement, Stmt};

use crate::{lexer::token::{EXP_IDENT, EXP_KEYWORD, Token, TokenExpectation}, parse::{Parser, Res}, tComb, tExp};

pub const EXP_ENDL: TokenExpectation = tExp!(Semicolon);

impl<'a> Parser<'a> {
    pub const EXP_STMT_STARTER: TokenExpectation = tComb!("Keyword | lvalue", EXP_KEYWORD, EXP_IDENT);
    pub const EXP_ENDL: TokenExpectation = tExp!(Semicolon);

    pub fn parse_stmt(&mut self) -> Res<Statement> {
        let t = self.expect_peek(Self::EXP_STMT_STARTER)?;

        match t.token() {
            Token::Def => {
                self.next()?;  // consume `t`

                // ugly way of this...
                let _id = self.expect(EXP_IDENT)?;
                let Token::Ident(id) = _id.token()
                    else { unreachable!() };

                // let lvalue = self.parse_lvalue()?;
                self.expect(tExp!(Equals))?;
                let rvalue = self.parse_expr()?;

                return Ok(Statement {
                    inner: Box::new(Stmt::Define(id.to_string(), rvalue)),
                    at: *t.position()
                })
            },
            Token::Return => {
                self.next()?;  // consume `t`

                if *self.peek()?.token() == Token::Scope {
                    self.next()?;
                    return Ok(Statement {
                        inner: Box::new(Stmt::ReturnScope),
                        at: *t.position()
                    });
                }

                else {
                    return Ok(Statement {
                        at: *t.position(),
                        inner: Box::new(Stmt::Return(self.parse_expr()?))
                    })
                }
            },
            Token::Break => {
                self.next()?;  // consume `t`

                return Ok(Statement {
                    at: *t.position(),
                    inner: Box::new(Stmt::Break),
                })
            },
            Token::Continue => {
                self.next()?;  // consume `t`

                return Ok(Statement {
                    at: *t.position(),
                    inner: Box::new(Stmt::Continue),
                })
            },
            Token::Loop => return self.parse_loop(),
            Token::If => return self.parse_if(),
            Token::Do => {
                self.next()?;  // consume `t`

                let expr = self.parse_expr()?;
                return Ok(Statement {
                    at: *t.position(),
                    inner: Box::new(Stmt::Expr(expr))
                })
            }
            Token::Ident(_) => {
                let lv = self.parse_lvalue()?;

                self.expect(tExp!(Equals))?;

                let right = self.parse_expr()?;

                return Ok(Statement {
                    at: lv.pos,
                    inner: Box::new(Stmt::Assign(lv, right))
                });
            }
            _ => unreachable!()
        }
    }

    // const BLOCK_ITEM: TokenExpectation = tComb!(
    //     "RSquirly | EXP_STMT_START",
    //     Parser::EXP_STMT_START,
    //     tExp!(RSquirly)
    // );
    // pub fn parse_block(&mut self) -> Res<Vec<Statement>> {
    //     self.expect(tExp!(LSquirly))?;
    //     let mut stmts = Vec::new();
    //     loop {
    //         let sp = self.expect_peek(BLOCK_ITEM)?;
    //         match sp.token() {
    //             Token::RSquirly => break,
    //             _ => {
    //                 let s = self.parse_stmt()?;
    //                 stmts.push(s);
    //                 self.expect(EXP_ENDL)?;
    //             }
    //         }
    //     }
    //     Ok(stmts)
    // }

    pub fn parse_block(&mut self) -> Res<Vec<Statement>> {
        self.expect(tExp!(LSquirly))?;

        let mut stmts = Vec::new();

        loop {
            if *self.peek()?.token() == Token::RSquirly {
                self.next()?; // consume it
                break;
            }

            let s = self.parse_stmt()?;
            stmts.push(s);

            self.expect(EXP_ENDL)?;
        }

        Ok(stmts)
    }
}