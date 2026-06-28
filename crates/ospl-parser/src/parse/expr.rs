use std::collections::HashMap;

use ospl_common::ast::{Expr, Expression, LV, LValue, Literal, UType, ops::{BinaryOp, BinaryOpType, UnaryOp, UnaryOpType}};

use crate::{lexer::token::{Span, Token, TokenExpectation, exp_ident}, parse::{PE, Parser, Res, macros::{self, MacroControl, MacroExpr, MacroStatement}}, tComb, tExp};

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

            Token::LSquirly => {
                let b = self.parse_block()?;
                return Ok(Expression {
                    at: *t.position(),
                    inner: Box::new(Expr::Block(b))
                })
            }

            Token::Macro => {
                self.next()?;

                let (_, Token::Ident(name)) = self.expect(exp_ident())?.destructure()
                else { unreachable!() };

                let Some(m) = self.local_macros.last()
                else { return Err(PE::MacroCallWithNoMacrosToCall) };

                let Some(mac) = m.get(&name).cloned()
                else { return Err(PE::NoSuchMacro(name)) };

                let mut mac_locals: HashMap<String, Vec<Span>> = HashMap::new();
                fn parse_macro_construct<'a>(p: &mut Parser<'a>, t: &macros::MacroConstructKind) -> Res<Vec<Span>> {
                    let toks =  match t {
                        macros::MacroConstructKind::Expr => p.parse_with_tokens(Parser::parse_expr)?.1,
                        macros::MacroConstructKind::Atom => p.parse_with_tokens(Parser::parse_atom)?.1,
                        macros::MacroConstructKind::Stmt => p.parse_with_tokens(Parser::parse_stmt)?.1,
                        macros::MacroConstructKind::Ident => {
                            let x = p.expect(exp_ident())?;
                            &[x]
                        },
                        macros::MacroConstructKind::Type => p.parse_with_tokens(Parser::parse_type)?.1,
                        macros::MacroConstructKind::SingleToken => p.parse_with_tokens(Parser::next)?.1,
                        // macros::MacroConstructKind::SpecificTokens(tt) => p.parse_with_tokens(|p| {
                        //     p.expect(TokenExpectation {
                        //         label: "???",
                        //         matches: Box::new(|t| { tt.contains(t) })
                        //     })?;

                        //     Ok(())
                        // })?.1
                        macros::MacroConstructKind::SpecificTokens(_) => unimplemented!()
                    };

                    tracing::debug!("parsed construct {t:?}: {toks:?}");

                    return Ok(toks.to_vec());
                }

                fn parse_macro_expr<'a>(
                    p: &mut Parser<'a>,
                    mac_locals: &HashMap<String, Vec<Span>>,
                    expr: &macros::MacroExpr
                ) -> Res<Vec<Span>> {
                    let spans = match expr {
                        macros::MacroExpr::Parse(c) => {
                            let x = parse_macro_construct(p, &c)?;
                            x
                        },
                        macros::MacroExpr::Var(v) => {
                            let Some(toks) = mac_locals.get(v)
                            else { return Err(PE::NoSuchMacroInput(v.to_owned())) };

                            toks.to_owned()
                        },
                        macros::MacroExpr::Literal(tok) => {
                            let pos = p.peek()?.destructure().0;
                            vec![Span::new(pos, tok.to_owned())]
                        },
                        macros::MacroExpr::Template(t) => {
                            let mut spns = Vec::new();
                            for thing in t {
                                spns.extend(parse_macro_expr(p, mac_locals, thing)?);
                            }

                            spns
                        },
                        macros::MacroExpr::Peek(c) => {
                            let mut subparser = p.clone();
                            let x = parse_macro_construct(&mut subparser, &c)?;
                            x
                        }
                    };


                    return Ok(spans)
                }

                fn run_macro_stmts<'a>(
                    p: &mut Parser<'a>,
                    mac_locals: &mut HashMap<String, Vec<Span>>,
                    v: &[MacroStatement],
                    output: &mut Vec<Span>
                ) -> Res<MacroControl>
                {
                    fn macro_if<'a>(
                        p: &mut Parser<'a>, mac_locals: &mut HashMap<String, Vec<Span>>,
                        cond: &MacroExpr, expected_toks: &MacroExpr, v: &[MacroStatement],
                        output: &mut Vec<Span>, invert: bool,
                    ) -> Res<MacroControl>
                    {
                        let tokens = parse_macro_expr(p, mac_locals, cond)?;
                        let expected_toks = parse_macro_expr(p, mac_locals, expected_toks)?;
                        tracing::debug!("macro testing: {tokens:?} == {expected_toks:?}");
                        let eq =
                            if invert { tokens != expected_toks }
                            else { tokens == expected_toks };

                        if eq {
                            tracing::debug!("macro if statement succeeded {tokens:?} == {expected_toks:?}");
                            return run_macro_stmts(p, mac_locals, &v, output);
                        } else {
                            tracing::debug!("macro if statement failed: {tokens:?} != {expected_toks:?}");
                            return Ok(MacroControl::Default)
                        }  
                    }

                    for stmt in v {
                        match stmt {
                            macros::MacroStatement::Define(var, expr) => {
                                let spans = parse_macro_expr(p, &mac_locals, &expr)?;
                                mac_locals.insert(var.to_owned(), spans);
                            },
                            macros::MacroStatement::Emit(expr) => {
                                let spans = parse_macro_expr(p, &mac_locals, &expr)?;
                                output.extend(spans);
                            },
                            macros::MacroStatement::Loop(l) => {
                                p.expect(tExp!(LBracket))?;
                                loop {
                                    if let Token::RBracket = p.peek()?.token() {
                                        p.next()?;
                                        break;
                                    }

                                    let mut out2 = Vec::new();
                                    if let MacroControl::BreakLoop = run_macro_stmts(p, mac_locals, l, &mut out2)? {
                                        tracing::debug!("breaking out of loop");
                                        break;
                                    }

                                    output.extend(out2);
                                }
                            },
                            MacroStatement::If(cond, expected_toks, v) => {
                                return macro_if(p, mac_locals, cond, expected_toks, v, output, false);
                            },
                            MacroStatement::IfNot(cond, expected_toks, v) => {
                                return macro_if(p, mac_locals, cond, expected_toks, v, output, true);
                            },
                            MacroStatement::Do(expr) => {
                                parse_macro_expr(p, mac_locals, expr)?;
                            }
                            MacroStatement::BreakLoop => return Ok(MacroControl::BreakLoop)
                        }
                    };

                    return Ok(MacroControl::Default)
                }

                let mut output = Vec::new();

                run_macro_stmts(self, &mut mac_locals, &mac.body, &mut output)?;

                let mut p = Parser::new(&output);
                match p.parse_expr() {
                    Ok(e) => return Ok(e),
                    Err(e) => return Err(PE::MacroInvocationError(Box::new(e)))
                }
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
            tExp!(LParen, LSquirly, Foreign, Macro),
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
            let span = self.peek()?;  // here

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

            else if let Token::Try = span.token() {
                panic!("reserved keywords");
            }

            else if *span.token() == Token::LBracket {
                // function specialization
                let map = self.parse_nominal_application()?;
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
        let initial_span = self.expect(exp_ident())?;
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

                    let span = self.expect(exp_ident())?;
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