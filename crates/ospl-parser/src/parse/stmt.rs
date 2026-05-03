use ospl_common::ast::{Statement, Stmt};

use crate::{lexer::token::{EXP_IDENT, EXP_KEYWORD, Token, TokenExpectation}, parse::{Parser, Res}, tExp};

pub const EXP_ENDL: TokenExpectation = tExp!(Semicolon);

impl<'a> Parser<'a> {
    pub const EXP_STMT_START: TokenExpectation = EXP_KEYWORD;
    pub const EXP_ENDL: TokenExpectation = tExp!(Semicolon);

    pub fn parse_stmt(&mut self) -> Res<Statement> {
        let t = self.expect(Self::EXP_STMT_START)?;

        match t.token() {
            Token::Def => {
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
                return Ok(Statement {
                    at: *t.position(),
                    inner: Box::new(Stmt::Break),
                })
            },
            Token::Continue => {
                return Ok(Statement {
                    at: *t.position(),
                    inner: Box::new(Stmt::Continue),
                })
            },
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