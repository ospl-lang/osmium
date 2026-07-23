use std::{collections::BTreeMap, sync::Arc};

use ospl_common::ast::{Expr, Expression, LV, LValue, Literal, UType, ops::{BinaryOp, BinaryOpType, UnaryOp, UnaryOpType}};

use crate::{lexer::token::{Span, Token, TokenExpectation, exp_ident}, parse::{PE, Parser, Res}, tComb, tExp};

// pub const EXP_STRING_LITERAL: TokenExpectation = TokenExpectation {
//     matches: |t| matches!(t, Token::StringLit(_)),
//     label: "String literal"
// };

pub fn exp_literal_starter() -> TokenExpectation {
    TokenExpectation {
        matches: Box::new(|t| -> bool {
            matches!(t,
                Token::StringLit(_) |
                Token::Integer(_) |
                Token::AddressLiteral(_) |
                Token::Float(_) |
                Token::Char(_) |
                Token::Fn |
                Token::True |
                Token::False |
                Token::Nul |
                Token::Undefined |
                Token::ListT
            )
        }),
        label: "literal starter (StringLit, Integer, Float, Fn, True or False)"
    }
}

pub fn exp_binary_operation() -> TokenExpectation {
    tExp!(
        Plus, Dash, Star, Slash, Percent, IsEqual, IsNotEqual, GreaterThanEqual, LessThanEqual, RAngle, LAngle,
        Question
    )
}

pub fn exp_unary_operation() -> TokenExpectation {
    tExp!(Increment, Decrement, Atsign, LogicNot, Copy)
}

impl<'a> Parser<'a> {
    pub fn parse_atom(&mut self) -> Res<Expression> {
        let t = self.expect_peek(Self::exp_expr_starter())?;

        match t.token() {
            Token::Integer(_) |
            Token::Float(_) |
            Token::AddressLiteral(_) |
            Token::Char(_) |
            Token::True |
            Token::False |
            Token::Undefined |
            Token::Nul |
            Token::Ident(_) => {
                let t = self.expect(exp_literal_starter())?.destructure();

                Ok(match t.1 {
                    Token::Integer(i) => Expression {
                        at: t.0,
                        inner: Box::new(Expr::Literal(Literal::Int(i))),
                        file: Arc::clone(&self.filename),
                    },

                    Token::AddressLiteral(u) => Expression {
                        at: t.0,
                        inner: Box::new(Expr::Literal(Literal::Address(u))),
                        file: Arc::clone(&self.filename),
                    },

                    Token::Float(f) => Expression {
                        at: t.0,
                        inner: Box::new(Expr::Literal(Literal::Float(f))),
                        file: Arc::clone(&self.filename),
                    },

                    Token::Char(c) => Expression {
                        at: t.0,
                        inner: Box::new(Expr::Literal(Literal::Char(c))),
                        file: Arc::clone(&self.filename),
                    },

                    Token::True => Expression {
                        at: t.0,
                        inner: Box::new(Expr::Literal(Literal::Bool(true))),
                        file: Arc::clone(&self.filename),
                    }, 

                    Token::False => Expression {
                        at: t.0,
                        inner: Box::new(Expr::Literal(Literal::Bool(false))),
                        file: Arc::clone(&self.filename),
                    },

                    Token::Nul => Expression {
                        at: t.0,
                        inner: Box::new(Expr::Literal(Literal::Nul)),
                        file: Arc::clone(&self.filename),
                    },

                    Token::Undefined => Expression {
                        at: t.0,
                        inner: Box::new(Expr::Literal(Literal::Undefined)),
                        file: Arc::clone(&self.filename),
                    },

                    Token::Ident(i) => Expression {
                        at: t.0,
                        inner: Box::new(Expr::LValue(LValue {
                            inner: Box::new(LV::Variable(i)),
                            at: t.0,
                            file: Arc::clone(&self.filename),
                        })),
                        file: Arc::clone(&self.filename),
                    },

                    _ => unreachable!(),
                })
            },

            Token::StringLit(s) => {
                self.next()?;
                return Ok(Expression {
                    at: *t.position(),
                    inner: Box::new(Expr::Literal(Literal::Str(s.to_string()))),
                    file: Arc::clone(&self.filename),
                });
            }

            Token::ListT => {
                self.next()?;

                // get the type
                let lty = self.parse_type()?;

                self.expect(tExp!(LSquirly))?;

                let mut exprs = Vec::new();
                loop {
                    let span = self.expect_peek(tComb!(
                        "RBracket | Expr",
                        tExp!(RSquirly),
                        exp_literal_starter(),
                    ))?;

                    match span.token() {
                        Token::RSquirly => {
                            self.next()?;
                            break;
                        },
                        _ => {
                            let e = self.parse_expr()?;
                            exprs.push(e);
                        }
                    }
                }

                return Ok(Expression {
                    at: *t.position(),
                    inner: Box::new(Expr::Literal(Literal::List(lty, exprs))),
                    file: Arc::clone(&self.filename),
                })
            }

            Token::Fn => Ok(Expression {
                at: *t.position(),
                inner: Box::new(Expr::Literal(Literal::Function(self.parse_function_literal()?))),
                    file: Arc::clone(&self.filename),
            }),

            Token::LParen => {
                self.expect(tExp!(LParen))?;
                let expr = self.parse_expr()?;
                self.expect(tExp!(RParen))?;
                Ok(expr)
            },

            Token::LSquirly => {
                let b = self.parse_block()?;
                return Ok(Expression {
                    at: *t.position(),
                    inner: Box::new(Expr::Block(b)),
                    file: Arc::clone(&self.filename),
                })
            }

            Token::Foreign => return self.parse_foreign_expr(),

            _ => unreachable!()
        }
    }

    pub fn exp_expr_starter() -> TokenExpectation {
        tComb!(
            "start of LValue | start of atom | foreign",
            exp_ident(),
            exp_literal_starter(),
            tExp!(LParen, LSquirly, Foreign),
        )
    }

    /// A primary is either:
    /// - an LValue
    /// - an atom
    pub fn parse_primary(&mut self) -> Res<Expression> {
        let span = self.expect_peek(Self::exp_expr_starter())?;
        let a1 = match span.token() {
            Token::Ident(_) => {
                let lv = self.parse_lvalue()?;
                Expression {
                    at: lv.at,
                    inner: Box::new(Expr::LValue(lv)),
                    file: Arc::clone(&self.filename),
                }
            },

            // must be atom starter now
            _ => self.parse_atom()?
        };

        return Ok(a1)
    }

    /// Returns the args to the call
    pub fn parse_fn_call_args(&mut self) -> Res<(Vec<Expression>, BTreeMap<String, Expression>)> {
        self.expect(tExp!(LParen))?;
        let mut args = Vec::new();
        let mut named_args = BTreeMap::new();
        loop {
            if *self.peek()?.token() == Token::RParen {
                self.next()?;
                break;
            }

            else if *self.peek()?.token() == Token::Def {
                self.next()?;
                let id = self.parse_ident()?;
                self.expect(tExp!(Equals))?;
                let expr = self.parse_expr()?;

                named_args.insert(id, expr);
            }

            else {
                let e = self.parse_expr()?;
                args.push(e);
            }
        }

        return Ok((args, named_args))
    }

    /// Returns the args to the call
    pub fn parse_nominal_application(&mut self) -> Res<Vec<(UType, UType)>> {
        self.expect(tExp!(LBracket))?;
        let mut map = Vec::new();
        loop {
            if *self.peek()?.token() == Token::RBracket {
                self.next()?;
                break;
            }

            let t1 = self.parse_type()?;

            self.expect(tExp!(Colon))?;

            let t2 = self.parse_type()?;

            map.push((t1, t2));
        }

        return Ok(map)
    }

    /// An expression is either:
    /// - a primary
    /// - or a chained binary operation on a primary.
    /// - a function call
    /// - a type cast
    /// 
    /// THIS DOES NOT FOLLOW BEDMAS.
    pub fn parse_expr(&mut self) -> Res<Expression> {
        let mut a1 = self.parse_primary()?;
        loop {
            let span = self.peek();  // here
            if let Err(PE::EOF) = span {
                break;
            }

            let span = span?;

            // parse operations
            if (exp_binary_operation().matches)(span.token()) {
                self.next()?;

                let optype = token_to_binaryop(&span.token());

                let a2 = self.parse_primary()?;
                a1 = Expression {
                    at: a1.at,
                    inner: Box::new(Expr::BinaryOp(BinaryOp {
                        left: a1,
                        right: a2,
                        kind: optype
                    })),
                    file: Arc::clone(&self.filename),
                };
            }

            else if (exp_unary_operation().matches)(span.token()) {
                self.next()?;
                
                let optype = match &span.token() {
                    Token::Increment => UnaryOpType::Increment,
                    Token::Decrement => UnaryOpType::Decrement,
                    Token::Atsign => UnaryOpType::Atsign,
                    Token::LogicNot => UnaryOpType::LogicNot,
                    Token::Copy => UnaryOpType::Copy,
                    _ => unreachable!()
                };

                a1 = Expression {
                    at: a1.at,
                    inner: Box::new(Expr::UnaryOp(UnaryOp {
                        expr: a1,
                        kind: optype
                    })),
                    file: Arc::clone(&self.filename),
                };
            }

            else if let Token::As = span.token() {
                self.next()?; 
                let t = self.parse_type()?;

                a1 = Expression {
                    at: a1.at,
                    inner: Box::new(Expr::Cast(a1, t)),
                    file: Arc::clone(&self.filename),
                }
            }

            else if let Token::Try = span.token() {
                panic!("reserved keywords");
            }

            else if *span.token() == Token::LBracket {
                // function specialization
                let map = self.parse_nominal_application()?;
                a1 = Expression {
                    at: a1.at,
                    inner: Box::new(Expr::Apply(a1, map)),
                    file: Arc::clone(&self.filename),
                };
            }

            if *span.token() == Token::LParen {
                // this is a fn call
                let (positional_args, named_args) = self.parse_fn_call_args()?;
                a1 = Expression {
                    at: a1.at,
                    inner: Box::new(Expr::Call {
                        left: a1,
                        args: positional_args,
                        named_args
                    }),
                    file: Arc::clone(&self.filename),
                };
            }

            // clearly we don't see any
            else {
                break;
            }
        };

        return Ok(a1)
    }

    pub fn parse_lvalue(&mut self) -> Res<LValue> {
        // get the initial variable
        let initial_span = self.expect(exp_ident())?;
        let Token::Ident(id) = initial_span.token()
            else { unreachable!() };

        let mut node = LValue {
            inner: Box::new(LV::Variable(id.to_string())),
            at: *initial_span.position(),
            file: Arc::clone(&self.filename),
        };

        while let Ok(t) = self.peek() {
            match t.token() {
                Token::Dot => {
                    let _ = self.next()?;  // consume `t`

                    let span = self.expect(exp_ident())?;
                    let Token::Ident(id) = span.token()
                        else { unreachable!() };

                    node = LValue {
                        inner: Box::new(LV::Property(node, id.to_string())),
                        at: *span.position(),
                        file: Arc::clone(&self.filename),
                    };
                },
                Token::Colon => {
                    let _ = self.next()?;  // consume `t` (a colon)

                    let a = self.parse_atom()?;
                    
                    if let Token::Comma = self.peek()?.token() {  // slicing
                        self.next()?;
                        let b = self.parse_atom()?;
                        node = LValue {
                            at: a.at,
                            inner: Box::new(LV::Slice(
                                Expression {
                                    at: a.at,
                                    inner: Box::new(Expr::LValue(node)),
                                    file: Arc::clone(&self.filename),
                                },
                                a, b,
                            )),
                            file: Arc::clone(&self.filename),
                        }
                    }

                    else {
                        node = LValue {
                            at: a.at,
                            inner: Box::new(LV::Index(
                                Expression {
                                    at: node.at,
                                    inner: Box::new(Expr::LValue(node)),
                                    file: Arc::clone(&self.filename),
                                },
                                a
                            )),
                            file: Arc::clone(&self.filename),
                        };
                    }
                }
                _ => break
            }
        };

        return Ok(node);
    }

    pub fn parse_with_tokens<R>(&mut self, p: fn(&mut Self) -> Res<R>) -> Res<(R, &[Span])> {
        let start = self.current_token;
        let data = p(self)?;
        let end = self.current_token;
        return Ok((data, &self.tokens[start..end]))
    }
}

pub fn token_to_binaryop(token: &Token) -> BinaryOpType {
    return match token {
        Token::Plus => BinaryOpType::Add,
        Token::Dash => BinaryOpType::Subtract,
        Token::Star => BinaryOpType::Multiply,
        Token::Slash => BinaryOpType::Divide,
        Token::Percent => BinaryOpType::Modulo,
        Token::IsEqual => BinaryOpType::Equals,
        Token::IsNotEqual => BinaryOpType::NotEquals,
        Token::LAngle => BinaryOpType::Lt,
        Token::RAngle => BinaryOpType::Gt,
        Token::LessThanEqual => BinaryOpType::Le,
        Token::GreaterThanEqual => BinaryOpType::Ge,
        Token::Question => BinaryOpType::Question,
        _ => unreachable!(),
    }
}