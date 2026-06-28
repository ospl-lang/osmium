use std::collections::HashMap;

use ospl_common::ast::{LValue, Statement, Stmt, UType, decl::{AliasDeclaration, Declaration}, ops::AssignOp};

use crate::{lexer::token::{exp_ident, Span, Token, TokenExpectation, exp_keyword}, parse::{Parser, Res, expr::{exp_binary_operation, token_to_binaryop}}, tComb, tExp};

pub fn exp_stmt_starter() -> TokenExpectation {
    return tComb!("Keyword | lvalue", exp_keyword(), exp_ident())
}

impl<'a> Parser<'a> {
    pub fn parse_stmt(&mut self) -> Res<Statement> {
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
                            inner: Box::new(Stmt::Define(Declaration {
                                name: id,
                                rhs: rvalue,
                            })),
                            at: *t.position()
                        });
                    },
                    Token::Colon => {
                        let rvalue = self.parse_type()?;

                        return Ok(Statement {
                            inner: Box::new(Stmt::DefineTypeAlias(AliasDeclaration {
                                name: id,
                                ty: rvalue,
                            })),
                            at: *t.position()
                        })
                    }
                    _ => unreachable!()
                }
            },
            Token::Let => {
                self.next()?;  // consume `t`
                // ugly way of this...
                let _id = self.expect(exp_ident())?;
                let (_, Token::Ident(id)) = _id.destructure()
                    else { unreachable!() };

                self.expect(tExp!(Colon))?;

                let typ = self.parse_type()?;

                return Ok(Statement {
                    at: *t.position(),
                    inner: Box::new(Stmt::DefineNominalTypeAlias(AliasDeclaration {
                        name: id,
                        ty: UType::Nominal(Box::new(typ))
                    }))
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
            Token::For => {
                if let Token::As = self.peekn(1)?.token() {
                    return self.parse_for_as()
                } else {
                    return self.parse_for_loop()
                }
            },
            Token::While => return self.parse_while(),
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

                if (exp_binary_operation().matches)(self.peek()?.token()) {
                    let op = self.next()?;
                    self.expect(tExp!(Equals))?;

                    // it's an assign op
                    return self.parse_assign_op_helper(lv, op);
                }
                self.expect(tExp!(Equals))?;

                let right = self.parse_expr()?;

                return Ok(Statement {
                    at: lv.at,
                    inner: Box::new(Stmt::Assign(lv, right))
                });
            },
            other => unreachable!("invalid token in stmt context: {other:?}")
        }
    }

    fn parse_assign_op_helper(&mut self, lv: LValue, op: Span) -> Res<Statement> {
        let rhs = self.parse_expr()?;

        return Ok(Statement {
            at: lv.at,
            inner: Box::new(Stmt::AssignOp(AssignOp {
                kind: token_to_binaryop(&op.token()),
                left: lv,
                right: rhs
            }))
        });
    }

    pub fn parse_block(&mut self) -> Res<Vec<Statement>> {
        self.expect(tExp!(LSquirly))?;

        let mut stmts = Vec::new();
        self.local_macros.push(HashMap::new());

        loop {
            if *self.peek()?.token() == Token::RSquirly {
                self.next()?;  // consume it
                break;
            }

            // `macro` construct
            if *self.peek()?.token() == Token::Macro
            {
                self.next()?;
                let (mname, mdef) = self.parse_macro_definition()?;
                self.expect(tExp!(Semicolon))?;

                let Some(macros) = self.local_macros.last_mut()
                else { panic!("wtf?"); };

                // println!("Registered macro: {mname}");

                macros.insert(mname, mdef);

                continue;
            }

            let s = self.parse_stmt()?;
            stmts.push(s);

            self.expect(tExp!(Semicolon))?;
        }

        self.local_macros.pop();

        return Ok(stmts)
    }

    /// Parses a top-level construct (usually a file)
    pub fn parse_tlc(&mut self) -> Res<Vec<Statement>> {
        let mut stmts = Vec::new();

        loop {
            if self.peek().is_err() {
                break;
            }

            let s = self.parse_stmt()?;
            stmts.push(s);

            self.expect(tExp!(Semicolon))?;
        }

        return Ok(stmts)
    }
}
