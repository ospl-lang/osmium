use std::sync::Arc;

use ospl_common::ast::{Expr, Expression, LV, LValue, Statement, Stmt, decl::Declaration, ops::{BinaryOp, BinaryOpType, UnaryOp, UnaryOpType}};

use crate::{lexer::token::{exp_ident, Token}, parse::{Parser, Res}, tExp};

impl<'a> Parser<'a> {
    pub fn parse_type_if(&mut self, unless: bool) -> Res<Statement> {
        let start = if unless {
            self.expect(tExp!(UnlessHash))?
        } else {
            self.expect(tExp!(IfHash))?
        };

        let id = self.parse_ident()?;

        // atsign notation
        let used_atsign = if *self.peek()?.token() == Token::Atsign {
            self.next()?;
            true
        } else { false };

        self.expect(tExp!(Colon))?;

        let ty = self.parse_type()?;
        self.expect(tExp!(Equals))?;
        let right = self.parse_expr()?;

        let mut yes = self.parse_block()?;

        // apply atsign notation
        if used_atsign {
            yes.push(Statement {
                at: *start.position(),
                inner: Box::new(Stmt::Define(Declaration {
                    name: id.clone(),
                    rhs: Expression {
                        at: *start.position(),
                        file: Arc::clone(&self.filename),
                        inner: Box::new(Expr::UnaryOp(UnaryOp {
                            kind: UnaryOpType::Atsign,
                            expr: Expression {
                                at: *start.position(),
                                file: Arc::clone(&self.filename),
                                inner: Box::new(Expr::LValue(LValue {
                                    at: *start.position(),
                                    file: Arc::clone(&self.filename),
                                    inner: Box::new(LV::Variable(id.clone()))
                                }))
                            }
                        }))
                    }
                })),
                file: Arc::clone(&self.filename),
                notes: String::new(),
            });
        }
        
        let no = match self.peek() {
            Ok(sp) if matches!(sp.token(), Token::Else) => {
                self.next()?;
                self.parse_block()?
            }
            _ => Vec::new()
        };

        if unless {
            // normal if
            return Ok(Statement {
                at: *start.position(),
                inner: Box::new(Stmt::TypeIf {
                    id,

                    no: yes,
                    yes: no,
                    
                    lhs: right,
                    typ: ty
                }),
                file: Arc::clone(&self.filename),
                notes: String::new(),
            })
        } else {
            // normal if
            return Ok(Statement {
                at: *start.position(),
                inner: Box::new(Stmt::TypeIf {
                    id, no, yes,
                    lhs: right,
                    typ: ty
                }),
                file: Arc::clone(&self.filename),
                notes: String::new(),
            })
        }
    }

    pub fn parse_if(&mut self, unless: bool) -> Res<Statement> {
        let start = if unless {
            self.expect(tExp!(Unless))?
        } else {
            self.expect(tExp!(If))?
        };

        let left = self.parse_expr()?;
        let yes = self.parse_block()?;

        let no = match self.peek() {
            Ok(sp) if matches!(sp.token(), Token::Else) => {
                self.next()?;
                self.parse_block()?
            }
            _ => Vec::new()
        };

        if unless {
            return Ok(Statement {
                at: *start.position(),
                inner: Box::new(Stmt::If(
                    left,
                    no,  // swap no and yes locations
                    yes,
                )),
                file: Arc::clone(&self.filename),
                notes: String::new(),
            })
        } else {
            return Ok(Statement {
                at: *start.position(),
                inner: Box::new(Stmt::If(
                    left,
                    yes,  // do not swap
                    no
                )),
                file: Arc::clone(&self.filename),
                notes: String::new(),
            })
        }
    }

    pub fn parse_loop(&mut self) -> Res<Statement> {
        let start = self.expect(tExp!(Loop))?;
        let x = self.parse_block()?;
        return Ok(Statement {
            at: *start.position(),
            inner: Box::new(Stmt::Loop(x)),
            notes: String::new(),
            file: Arc::clone(&self.filename),
        })
    }

    /// * Syntax: `for as IDENT = EXPR1 => EXPR2`
    /// * Example: `for as x = iter.List(l) => x()`
    pub fn parse_for_as(&mut self) -> Res<Statement> {
        let thing = self.expect(tExp!(For))?;
        self.expect(tExp!(As))?;

        // id
        let id = self.parse_ident()?;

        self.expect(tExp!(Equals))?;

        // expr
        let expr1 = self.parse_expr()?;

        self.expect(tExp!(Equals))?;

        let expr2 = self.parse_expr()?;

        let user_body = self.parse_block()?;
        // All this crap just expands to this:
        // ```
        // if true {
        //      def IDENT = EXPR1;
        //      loop {
        //          def IDENT = EXPR2;
        //          if (IDENT as bool)! { break; };
        //          BODY;
        //      };
        // };
        // ```

        let mut body = vec![
            Statement {
                at: *thing.position(),
                inner: Box::new(Stmt::Define(Declaration {
                    name: id.clone(),
                    rhs: expr2,
                })),
                file: Arc::clone(&self.filename),
                notes: String::new(),
            },
            Statement {
                at: *thing.position(),
                inner: Box::new(Stmt::If(
                    Expression {
                        at: *thing.position(),
                        inner: Box::new(Expr::UnaryOp(UnaryOp {
                            expr: Expression {
                                file: Arc::clone(&self.filename),
                                at: *thing.position(),
                                inner: Box::new(Expr::LValue(LValue {
                                    at: *thing.position(),
                                    inner: Box::new(LV::Variable(id.clone())),
                                    file: Arc::clone(&self.filename),
                                }))
                            },
                            kind: UnaryOpType::LogicNot
                        })),
                        file: Arc::clone(&self.filename),
                    },
                    vec![Statement {
                        file: Arc::clone(&self.filename),
                        at: *thing.position(),
                        inner: Box::new(Stmt::Break),
                        notes: String::new(),
                    }],
                    vec![]
                )),
                notes: String::new(),
                file: Arc::clone(&self.filename),
            },
        ];
        body.extend_from_slice(&user_body);

        let code = vec![
            Statement {
                at: *thing.position(),
                inner: Box::new(Stmt::Define(Declaration {
                    name: id.clone(),
                    rhs: expr1
                })),
                notes: String::new(),
                file: Arc::clone(&self.filename),
            },
            Statement {
                at: *thing.position(),
                inner: Box::new(Stmt::Loop(body)),
                notes: String::new(),
                file: Arc::clone(&self.filename),
            }
        ];
        
        return Ok(Statement {
            at: *thing.position(),
            inner: Box::new(Stmt::If(
                Expression {
                    at: *thing.position(),
                    inner: Box::new(Expr::Literal(ospl_common::ast::Literal::Bool(true))),
                    file: Arc::clone(&self.filename),
                },
                code,
                vec![]  // unreachable
            )),
            notes: String::new(),
            file: Arc::clone(&self.filename),
        })
    }

    /// Syntax: `for x  y`
    pub fn parse_for_loop(&mut self) -> Res<Statement> {
        let thing = self.expect(tExp!(For))?;
        
        // id
        let (_, Token::Ident(id)) = self.expect(exp_ident())?.destructure()
        else { unreachable!() };

        // expr
        let expr = self.parse_expr()?;

        // body
        let user_body = self.parse_block()?;
        let mut body = vec![
            Statement {
                at: *thing.position(),
                inner: Box::new(Stmt::Define(Declaration {
                    name: id.clone(),
                    rhs: expr
                })),
                notes: String::new(),
                file: Arc::clone(&self.filename),
            },
            Statement {
                at: *thing.position(),
                inner: Box::new(Stmt::If(
                    Expression {
                        at: *thing.position(),
                        inner: Box::new(Expr::BinaryOp(BinaryOp {
                            kind: BinaryOpType::Equals,
                            left: Expression {
                                at: *thing.position(),
                                inner: Box::new(Expr::LValue(LValue {
                                    at: *thing.position(),
                                    inner: Box::new(LV::Variable(id.clone())),
                                    file: Arc::clone(&self.filename),
                                })),
                                file: Arc::clone(&self.filename),
                            },
                            right: Expression {
                                at: *thing.position(),
                                inner: Box::new(Expr::Literal(ospl_common::ast::Literal::Undefined)),
                                file: Arc::clone(&self.filename),
                            }
                        })),
                        file: Arc::clone(&self.filename),
                    },
                    vec![
                        Statement {
                            at: *thing.position(),
                            inner: Box::new(Stmt::Break),
                            notes: String::new(),
                            file: Arc::clone(&self.filename),
                        }
                    ],
                    vec![]
                )),
                notes: String::new(),
                file: Arc::clone(&self.filename),
            }
        ];

        body.extend_from_slice(&user_body);

        return Ok(Statement {
            at: *thing.position(),
            inner: Box::new(Stmt::Loop(body)),
            notes: String::new(),
            file: Arc::clone(&self.filename),
        });
    }

    pub fn parse_while(&mut self) -> Res<Statement> {
        let thing = self.expect(tExp!(While))?;
        let expr = self.parse_expr()?;
        let body = self.parse_block()?;

        let mut loop_body = vec![
            Statement {
                at: *thing.position(),
                inner: Box::new(Stmt::If(
                    expr,
                    vec![],
                    vec![Statement {
                        at: *thing.position(),
                        inner: Box::new(Stmt::Break),
                        notes: String::new(),
                        file: Arc::clone(&self.filename),
                    }]
                )),
                notes: String::new(),
                file: Arc::clone(&self.filename),
            }
        ];
        loop_body.extend(body);

        return Ok(Statement {
            at: *thing.position(),
            inner: Box::new(Stmt::Loop(loop_body)),
            notes: String::new(),
            file: Arc::clone(&self.filename),
        })
    }
}