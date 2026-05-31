use std::collections::HashMap;

use ospl_common::ast::{Expr, Expression, LV, LValue, Literal, UType, ops::{BinaryOp, BinaryOpType, UnaryOp, UnaryOpType}};

use crate::{lexer::token::{EXP_IDENT, Token, TokenExpectation}, parse::{Parser, Res}, tComb, tExp};

// pub const EXP_STRING_LITERAL: TokenExpectation = TokenExpectation {
//     matches: |t| matches!(t, Token::StringLit(_)),
//     label: "String literal"
// };

pub const EXP_LITERAL_STARTER: TokenExpectation = TokenExpectation {
    matches: |t| -> bool {
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
    },
    label: "literal starter (StringLit, Integer, Float, Fn, True or False)"
};

pub const EXP_BINARY_OPERATION: TokenExpectation = tExp!(
    Plus, Dash, Star, Slash, Percent, IsEqual, IsNotEqual, GreaterThanEqual, LessThanEqual, RAngle, LAngle,
    Question
);

pub const EXP_UNARY_OPERATION: TokenExpectation = tExp!(Increment, Decrement);

impl<'a> Parser<'a> {
    pub fn parse_atom(&mut self) -> Res<Expression> {
        let t = self.expect_peek(tComb!(
            "EXP_LITERAL_STARTER | LParen | Foreign",
            tExp!(LParen, Foreign),
            EXP_LITERAL_STARTER
        ))?;

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
                let t = self.expect(EXP_LITERAL_STARTER)?.destructure();

                Ok(match t.1 {
                    Token::Integer(i) => Expression {
                        at: t.0,
                        inner: Box::new(Expr::Literal(Literal::Int(i))),
                    },

                    Token::AddressLiteral(u) => Expression {
                        at: t.0,
                        inner: Box::new(Expr::Literal(Literal::Address(u))),
                    },

                    Token::Float(f) => Expression {
                        at: t.0,
                        inner: Box::new(Expr::Literal(Literal::Float(f))),
                    },

                    Token::Char(c) => Expression {
                        at: t.0,
                        inner: Box::new(Expr::Literal(Literal::Char(c))),
                    },

                    Token::True => Expression {
                        at: t.0,
                        inner: Box::new(Expr::Literal(Literal::Bool(true))),
                    }, 

                    Token::False => Expression {
                        at: t.0,
                        inner: Box::new(Expr::Literal(Literal::Bool(false))),
                    },

                    Token::Nul => Expression {
                        at: t.0,
                        inner: Box::new(Expr::Literal(Literal::Nul)),
                    },

                    Token::Undefined => Expression {
                        at: t.0,
                        inner: Box::new(Expr::Literal(Literal::Undefined)),
                    },

                    Token::Ident(i) => Expression {
                        at: t.0,
                        inner: Box::new(Expr::LValue(LValue {
                            inner: Box::new(LV::Variable(i)),
                            at: t.0,
                        })),
                    },

                    _ => unreachable!(),
                })
            },

            Token::StringLit(s) => {
                self.next()?;
                return Ok(Expression {
                    at: *t.position(),
                    inner: Box::new(Expr::Literal(Literal::Str(s.to_string()))),
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
                        EXP_LITERAL_STARTER
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
                })
            }

            Token::Fn => Ok(Expression {
                at: *t.position(),
                inner: Box::new(Expr::Literal(Literal::Function(self.parse_function_literal()?))),
            }),

            Token::LParen => {
                self.expect(tExp!(LParen))?;
                let expr = self.parse_expr()?;
                self.expect(tExp!(RParen))?;
                Ok(expr)
            },

            Token::Foreign => return self.parse_foreign_expr(),

            _ => unreachable!()
        }
    }

    pub const EXP_EXPR_STARTER: TokenExpectation = tComb!(
        "start of LValue | start of atom | foreign",
        EXP_IDENT,
        EXP_LITERAL_STARTER,
        tExp!(LParen, Foreign),
    );

    /// A primary is either:
    /// - an LValue
    /// - an atom
    pub fn parse_primary(&mut self) -> Res<Expression> {
        let span = self.expect_peek(Self::EXP_EXPR_STARTER)?;
        let a1 = match span.token() {
            Token::Ident(_) => {
                let lv = self.parse_lvalue()?;
                Expression {
                    at: lv.at,
                    inner: Box::new(Expr::LValue(lv)),
                }
            },

            // must be atom starter now
            _ => self.parse_atom()?
        };

        return Ok(a1)
    }

    /// Returns the args to the call
    pub fn parse_fn_call_args(&mut self) -> Res<Vec<Expression>> {
        self.expect(tExp!(LParen))?;
        let mut args = Vec::new();
        loop {
            if *self.peek()?.token() == Token::RParen {
                self.next()?;
                break;
            }

            let e = self.parse_expr()?;
            args.push(e);
        }

        return Ok(args)
    }

    /// Returns the args to the call
    pub fn parse_fn_specialization(&mut self) -> Res<HashMap<UType, UType>> {
        self.expect(tExp!(LSquirly))?;
        let mut map = HashMap::new();
        loop {
            if *self.peek()?.token() == Token::RSquirly {
                self.next()?;
                break;
            }

            let t1 = self.parse_type()?;

            self.expect(tExp!(Colon))?;

            let t2 = self.parse_type()?;

            map.insert(t1, t2);
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
            let span = self.peek()?;  // here

            // parse operations
            if (EXP_BINARY_OPERATION.matches)(span.token()) {
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
                };
            }

            else if (EXP_UNARY_OPERATION.matches)(span.token()) {
                self.next()?;
                
                let optype = match &span.token() {
                    Token::Increment => UnaryOpType::Increment,
                    Token::Decrement => UnaryOpType::Decrement,
                    Token::LogicNot => UnaryOpType::LogicNot,
                    _ => unreachable!()
                };

                a1 = Expression {
                    at: a1.at,
                    inner: Box::new(Expr::UnaryOp(UnaryOp {
                        expr: a1,
                        kind: optype
                    })),
                };
            }

            else if let Token::As = span.token() {
                self.next()?; 
                let t = self.parse_type()?;

                a1 = Expression {
                    at: a1.at,
                    inner: Box::new(Expr::Cast(a1, t))
                }
            }

            else if *span.token() == Token::LSquirly {
                // function specialization
                let map = self.parse_fn_specialization()?;
                a1 = Expression {
                    at: a1.at,
                    inner: Box::new(Expr::Apply(a1, map))
                };
            }

            if *span.token() == Token::LParen {
                // this is a fn call
                let args = self.parse_fn_call_args()?;
                a1 = Expression {
                    at: a1.at,
                    inner: Box::new(Expr::Call(a1, args)),
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
        let initial_span = self.expect(EXP_IDENT)?;
        let Token::Ident(id) = initial_span.token()
            else { unreachable!() };

        let mut node = LValue {
            inner: Box::new(LV::Variable(id.to_string())),
            at: *initial_span.position()
        };

        while let Ok(t) = self.peek() {
            match t.token() {
                Token::Dot => {
                    let _ = self.next()?;  // consume `t`

                    let span = self.expect(EXP_IDENT)?;
                    let Token::Ident(id) = span.token()
                        else { unreachable!() };

                    node = LValue {
                        inner: Box::new(LV::Property(node, id.to_string())),
                        at: *span.position()
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
                                    inner: Box::new(Expr::LValue(node))
                                },
                                a, b,
                            ))
                        }
                    }

                    else {
                        node = LValue {
                            at: a.at,
                            inner: Box::new(LV::Index(
                                Expression {
                                    at: node.at,
                                    inner: Box::new(Expr::LValue(node))
                                },
                                a
                            ))
                        };
                    }
                }
                _ => break
            }
        };

        return Ok(node);
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