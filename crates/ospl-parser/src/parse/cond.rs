use ospl_common::ast::{Expr, Expression, LV, LValue, Statement, Stmt, decl::Declaration, ops::{BinaryOp, BinaryOpType, UnaryOp, UnaryOpType}};

use crate::{lexer::token::{EXP_IDENT, Token, TokenExpectation}, parse::{Parser, Res}, tExp};

impl<'a> Parser<'a> {
    pub fn parse_if(&mut self) -> Res<Statement> {
        let start = self.expect(tExp!(If))?;

        let left = self.parse_expr()?;
        let yes = self.parse_block()?;

        let no = if let Token::Else = self.peek()?.token() {
            self.next()?;
            self.parse_block()?
        } else { Vec::new() };

        return Ok(Statement {
            at: *start.position(),
            inner: Box::new(Stmt::If(
                left,
                yes,
                no
            ))
        })
    }

    pub fn parse_loop(&mut self) -> Res<Statement> {
        let start = self.expect(tExp!(Loop))?;
        let x = self.parse_block()?;
        return Ok(Statement {
            at: *start.position(),
            inner: Box::new(Stmt::Loop(x)),
        })
    }

    /// * Syntax: `for as IDENT = EXPR1 => EXPR2`
    /// * Example: `for as x = iter.List(l) => x()`
    pub fn parse_for_as(&mut self) -> Res<Statement> {
        let thing = self.expect(tExp!(For))?;
        self.expect(tExp!(As))?;

        // id
        let (_, Token::Ident(id)) = self.expect(EXP_IDENT)?.destructure()
        else { unreachable!() };

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
                }))
            },
            Statement {
                at: *thing.position(),
                inner: Box::new(Stmt::If(
                    Expression {
                        at: *thing.position(),
                        inner: Box::new(Expr::UnaryOp(UnaryOp {
                            expr: Expression {
                                at: *thing.position(),
                                inner: Box::new(Expr::LValue(LValue {
                                    at: *thing.position(),
                                    inner: Box::new(LV::Variable(id.clone()))
                                }))     
                            },
                            kind: UnaryOpType::LogicNot
                        }))
                    },
                    vec![Statement {
                        at: *thing.position(),
                        inner: Box::new(Stmt::Break)
                    }],
                    vec![]
                ))  
            },
        ];
        body.extend_from_slice(&user_body);

        let code = vec![
            Statement {
                at: *thing.position(),
                inner: Box::new(Stmt::Define(Declaration {
                    name: id.clone(),
                    rhs: expr1
                }))
            },
            Statement {
                at: *thing.position(),
                inner: Box::new(Stmt::Loop(body))
            }
        ];
        
        return Ok(Statement {
            at: *thing.position(),
            inner: Box::new(Stmt::If(
                Expression {
                    at: *thing.position(),
                    inner: Box::new(Expr::Literal(ospl_common::ast::Literal::Bool(true)))
                },
                code,
                vec![]  // unreachable
            ))
        })
    }

    /// Syntax: `for x  y`
    pub fn parse_for_loop(&mut self) -> Res<Statement> {
        let thing = self.expect(tExp!(For))?;
        
        // id
        let (_, Token::Ident(id)) = self.expect(EXP_IDENT)?.destructure()
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
                }))
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
                                    inner: Box::new(LV::Variable(id.clone()))
                                }))
                            },
                            right: Expression {
                                at: *thing.position(),
                                inner: Box::new(Expr::Literal(ospl_common::ast::Literal::Undefined))
                            }
                        }))
                    },
                    vec![
                        Statement {
                            at: *thing.position(),
                            inner: Box::new(Stmt::Break)
                        }
                    ],
                    vec![]
                ))
            }
        ];

        body.extend_from_slice(&user_body);

        return Ok(Statement {
            at: *thing.position(),
            inner: Box::new(Stmt::Loop(body))
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
                        inner: Box::new(Stmt::Break)
                    }]
                ))
            }
        ];
        loop_body.extend(body);

        return Ok(Statement {
            at: *thing.position(),
            inner: Box::new(Stmt::Loop(loop_body))
        })
    }
}