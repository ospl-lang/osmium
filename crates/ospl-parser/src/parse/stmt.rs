use std::sync::Arc;

use ospl_common::ast::{LValue, Statement, Stmt, UType, decl::{AliasDeclaration, Declaration}, ops::AssignOp};

use crate::{lexer::token::{Span, Token, TokenExpectation, exp_ident, exp_keyword}, parse::{PE, Parser, Res, expr::{exp_binary_operation, token_to_binaryop}}, tComb, tExp};

pub fn exp_stmt_starter() -> TokenExpectation {
    return tComb!("Keyword | lvalue", exp_keyword(), exp_ident())
}

impl<'a> Parser<'a> {
    pub fn parse_stmt(&mut self) -> Res<Statement> {
        let notes = if let (_, Token::BlockComment(block)) = self.peek()?.destructure() {
            self.next()?;
            block
        } else { String::new() };

        let t = self.expect_peek(exp_stmt_starter())?;

        match t.token() {
            Token::Def => {
                self.next()?;  // consume `t`

                // ugly way of this...
                let _id = self.expect(exp_ident())?;
                let (_, Token::Ident(id)) = _id.destructure()
                    else { unreachable!() };

                let span = self.expect(tExp!(Equals, Colon))?;
                match span.token() {
                    Token::Equals => {
                        let rvalue = self.parse_expr()?;

                        return Ok(Statement {
                            file: Arc::clone(&self.filename),
                            inner: Box::new(Stmt::Define(Declaration {
                                name: id,
                                rhs: rvalue,
                            })),
                            at: *t.position(),
                            notes
                        });
                    },
                    Token::Colon => {
                        let rvalue = self.parse_type()?;

                        return Ok(Statement {
                            file: Arc::clone(&self.filename),
                            inner: Box::new(Stmt::DefineTypeAlias(AliasDeclaration {
                                name: id,
                                ty: rvalue,
                            })),
                            at: *t.position(),
                            notes
                        })
                    }
                    _ => unreachable!()
                }
            },
            Token::Distinct => {
                self.next()?;  // consume `t`
                // ugly way of this...
                let _id = self.expect(exp_ident())?;
                let (_, Token::Ident(id)) = _id.destructure()
                    else { unreachable!() };

                self.expect(tExp!(Colon))?;

                let typ = self.parse_type()?;

                return Ok(Statement {
                    file: Arc::clone(&self.filename),
                    at: *t.position(),
                    inner: Box::new(Stmt::DefineNominalTypeAlias(AliasDeclaration {
                        name: id,
                        ty: UType::Nominal(Box::new(typ))
                    })),
                    notes
                })
            },
            Token::Return => {
                self.next()?;  // consume `t`

                if *self.peek()?.token() == Token::Scope {
                    self.next()?;
                    return Ok(Statement {
                        file: Arc::clone(&self.filename),
                        inner: Box::new(Stmt::ReturnScope),
                        at: *t.position(),
                        notes
                    });
                }

                else {
                    return Ok(Statement {
                        file: Arc::clone(&self.filename),
                        at: *t.position(),
                        inner: Box::new(Stmt::Return(self.parse_expr()?)),
                        notes
                    })
                }
            },
            Token::Break => {
                self.next()?;  // consume `t`

                return Ok(Statement {
                    file: Arc::clone(&self.filename),
                    at: *t.position(),
                    inner: Box::new(Stmt::Break),
                    notes
                })
            },
            Token::Continue => {
                self.next()?;  // consume `t`

                return Ok(Statement {
                    file: Arc::clone(&self.filename),
                    at: *t.position(),
                    inner: Box::new(Stmt::Continue),
                    notes
                })
            },
            Token::For => {
                // if let Token::As = self.peekn(1)?.token() {
                    // return self.parse_for_as()
                // } else {
                return self.parse_for_loop()
                // }
            },
            Token::While => return self.parse_while(),
            Token::Loop => return self.parse_loop(),
            Token::If => return self.parse_if(),
            Token::Do => {
                self.next()?;  // consume `t`

                let expr = self.parse_expr()?;
                return Ok(Statement {
                    at: *t.position(),
                    file: Arc::clone(&self.filename),
                    inner: Box::new(Stmt::Expr(expr)),
                    notes
                })
            }
            Token::Ident(_) => {
                let lv = self.parse_lvalue()?;

                if (exp_binary_operation().matches)(self.peek()?.token()) {
                    let op = self.next()?;
                    self.expect(tExp!(Equals))?;

                    // it's an assign op
                    return self.parse_assign_op_helper(notes, lv, op);
                }
                self.expect(tExp!(Equals))?;

                let right = self.parse_expr()?;

                return Ok(Statement {
                    file: Arc::clone(&self.filename),
                    at: lv.at,
                    inner: Box::new(Stmt::Assign(lv, right)),
                    notes
                });
            },
            other => unreachable!("invalid token in stmt context: {other:?}")
        }
    }

    fn parse_assign_op_helper(&mut self, notes: String, lv: LValue, op: Span) -> Res<Statement> {
        let rhs = self.parse_expr()?;

        return Ok(Statement {
            file: Arc::clone(&self.filename),
            at: lv.at,
            inner: Box::new(Stmt::AssignOp(AssignOp {
                kind: token_to_binaryop(&op.token()),
                left: lv,
                right: rhs
            })),
            notes
        });
    }

    pub fn parse_block(&mut self) -> Res<Vec<Statement>> {
        self.expect(tExp!(LSquirly))?;

        let mut stmts = Vec::new();
        loop {
            if *self.peek()?.token() == Token::RSquirly {
                self.next()?;  // consume it
                break;
            }

            let s = self.parse_stmt()?;
            stmts.push(s);
        }

        return Ok(stmts)
    }

    /// Parses a top-level construct (usually a file)
    pub fn parse_tlc(&mut self) -> Res<Vec<Statement>> {
        let mut stmts = Vec::new();

        loop {
            // `macro` construct
            let peek = self.peek();
            if let Err(PE::EOF) = peek {
                break;
            }

            peek?;

            let s = self.parse_stmt();
            if let Err(PE::EOF) = s {
                break;
            }

            let s = s?;
            stmts.push(s);
        }

        return Ok(stmts)
    }
}
