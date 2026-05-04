use ospl_common::ast::{Expr, Expression, LV, LValue, Literal, ops::{BinaryOp, BinaryOpType}};

use crate::{lexer::token::{EXP_IDENT, Span, Token, TokenExpectation}, parse::{Parser, Res}, tComb, tExp};

pub const EXP_LITERAL_STARTER: TokenExpectation = TokenExpectation {
    matches: |t| -> bool {
        matches!(t,
            Token::StringLit(_) |
            Token::Integer(_) |
            Token::Float(_) |
            Token::Fn |
            Token::True |
            Token::False
        )
    },
    label: "literal starter (StringLit, Integer, Float, Fn, True or False)"
};

pub const EXP_OPERATION: TokenExpectation = tExp!(Plus, Dash, Star, Slash, Percent, IsEqual, GreaterThanEqual, LessThanEqual, RAngle, LAngle);

impl<'a> Parser<'a> {
    pub fn parse_atom(&mut self) -> Res<Expression> {
        let t = self.expect_peek(tComb!(
            "EXP_LITERAL_STARTER | LParen",
            tExp!(LParen),
            EXP_LITERAL_STARTER
        ))?;

        match t.token() {
            Token::Integer(_) |
            Token::Float(_) |
            Token::StringLit(_) |
            Token::True |
            Token::False |
            Token::Ident(_) => {
                let t = self.expect(EXP_LITERAL_STARTER)?.destructure();

                Ok(match t.1 {
                    Token::StringLit(s) => Expression {
                        at: t.0,
                        inner: Box::new(Expr::Literal(Literal::Str(s.to_string()))),
                    },

                    Token::Integer(i) => Expression {
                        at: t.0,
                        inner: Box::new(Expr::Literal(Literal::Int(i))),
                    },

                    Token::Float(f) => Expression {
                        at: t.0,
                        inner: Box::new(Expr::Literal(Literal::Float(f))),
                    },

                    Token::True => Expression {
                        at: t.0,
                        inner: Box::new(Expr::Literal(Literal::Bool(true))),
                    }, 

                    Token::False => Expression {
                        at: t.0,
                        inner: Box::new(Expr::Literal(Literal::Bool(false))),
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

            _ => unreachable!()
        }
    }

    pub const EXP_EXPR_STARTER: TokenExpectation = tComb!(
        "EXP_LVALUE_STARTER | EXP_ATOM_STARTER",
        EXP_IDENT,
        EXP_LITERAL_STARTER,
        tExp!(LParen),
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

    /// An expression is either:
    /// - a primary
    /// - or a chained binary operation on a primary.
    /// - a function call
    /// 
    /// THIS DOES NOT FOLLOW BEDMAS.
    pub fn parse_expr(&mut self) -> Res<Expression> {
        let mut a1 = self.parse_primary()?;
        loop {
            let span = self.peek()?;

            // parse operations
            if (EXP_OPERATION.matches)(span.token()) {
                self.next()?;

                let optype = span_to_binaryop(&span);

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
                _ => break
            }
        };

        return Ok(node);
    }
}

fn span_to_binaryop(span: &Span) -> BinaryOpType {
    return match span.token() {
        Token::Plus => BinaryOpType::Add,
        Token::Dash => BinaryOpType::Subtract,
        Token::Star => BinaryOpType::Multiply,
        Token::Slash => BinaryOpType::Divide,
        Token::Percent => BinaryOpType::Modulo,
        Token::IsEqual => BinaryOpType::Equals,
        Token::LAngle => BinaryOpType::Lt,
        Token::RAngle => BinaryOpType::Gt,
        Token::LessThanEqual => BinaryOpType::Le,
        Token::GreaterThanEqual => BinaryOpType::Ge,
        _ => unreachable!(),
    }
}