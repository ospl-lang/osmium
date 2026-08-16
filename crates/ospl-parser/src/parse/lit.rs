use std::collections::BTreeMap;

use ospl_common::ast::{ArgNaming, FunctionType, FunctionValue, Type, UType};
use crate::{lexer::token::{Token, TokenExpectation, exp_ident}, parse::{Parser, Res}, tComb, tExp};

impl<'a> Parser<'a> {
    pub fn parse_function_literal(&mut self) -> Res<FunctionValue> {
        self.expect(tExp!(Fn))?;

        // get the captures
        let mut captures = Vec::new();
        if let Token::LBracket = self.peek()?.token() {
            self.next()?;  // consume `[`
            loop {
                let id = self.expect(tComb!(
                    "identifier | RBracket",
                    tExp!(RBracket),
                    exp_ident()
                ))?;

                let (_, Token::Ident(id)) = id.destructure()
                else { break; };

                captures.push(id);
            }
        }

        // get the args using two paralel lists: very fucking stupid
        let mut arg_types = Vec::new();
        let mut named_args = BTreeMap::new();
        let mut arg_values = Vec::new();
        self.expect(tExp!(LParen))?;
        loop {
            let (_, token) = self.expect(exp_named_arg_member())?.destructure();
            match token {
                Token::Ident(i) => {
                    arg_values.push(ArgNaming {
                        name: i,
                    });

                    self.expect(tExp!(Colon))?;

                    arg_types.push(self.parse_type()?);
                }
                Token::Def => {
                    let id = self.parse_ident()?;

                    self.expect(tExp!(Colon))?;

                    let ty = self.parse_type()?;

                    named_args.insert(id, ty);
                }
                Token::RParen => break,
                _ => unreachable!()
            }
        }  // NOTE: we consumed RParen in the loop

        let ret = self.parse_function_return_type()?;

        let b = self.parse_block()?;
        return Ok(FunctionValue {
            ftype: FunctionType {
                args: arg_types,
                named_args,
                ret
            },
            block: b,
            args: arg_values,
            captures,
        })
    }

    pub fn parse_function_type(&mut self) -> Res<FunctionType<UType>> {
        self.expect(tExp!(Fn))?;

        // arg types
        self.expect(tExp!(LParen))?;
        let mut args = Vec::new();
        let mut named_args = BTreeMap::new();
        loop {
            let span = self.expect_peek(tComb!(
                "Begining of type | Def | RParen",
                exp_type_starter(),
                tExp!(Def, RParen)
            ))?;

            match span.token() {
                Token::Def => {
                    self.next()?;
                    let id = self.parse_ident()?;
                    self.expect(tExp!(Colon))?;
                    let ty = self.parse_type()?;

                    named_args.insert(id, ty);
                },
                Token::RParen => {
                    self.next()?;
                    break;
                },
                _ => {
                    // if it's not the end of the list
                    let t = self.parse_type()?;
                    args.push(t);
                }
            }
        }

        let ret = self.parse_function_return_type()?;

        return Ok(FunctionType {
            args,
            named_args,
            ret,
        });
    }

    pub fn parse_function_return_type(&mut self) -> Res<UType> {
        // get return type (defaults to nul)
        let ret = if let Token::Colon = self.peek()?.token() {
            self.next()?;  // take arrow
            self.parse_type()?
        } else {
            UType::Resolved(Type::Nul)
        };

        return Ok(ret)
    }

    pub fn parse_type_atom(&mut self, t: Token) -> Res<UType> {
        return Ok(match t {
            Token::LParen => {
                self.next()?;
                let t = self.parse_type()?;
                self.expect(tExp!(RParen))?;
                return Ok(t)
            },
            Token::Fn => UType::Function(Box::new(self.parse_function_type()?)),
            Token::Ident(i) => {self.next()?; UType::Typeof(i)},
            Token::Atsign => {
                self.next()?;  // consume the `@`
                // let (_, tok) = self.next()?.destructure();
                // let t = self.parse_type_atom(tok)?;
                let t = self.parse_type()?;
                UType::Returnof(Box::new(t))
            },
            Token::AnyT => {self.next()?; UType::Resolved(Type::Any)},

            /* primitives */
            Token::IntT => {self.next()?; UType::Resolved(Type::Int)},
            Token::FloatT => {self.next()?; UType::Resolved(Type::Float)},
            Token::StrT => {self.next()?; UType::Resolved(Type::Str)},
            Token::CharT => {self.next()?; UType::Resolved(Type::Char)},
            Token::BoolT => {self.next()?; UType::Resolved(Type::Bool)},
            Token::AddrT => {self.next()?; UType::Resolved(Type::Address)},
            Token::Undefined => {self.next()?; UType::Resolved(Type::Undefined)},
            Token::Nul => {self.next()?; UType::Resolved(Type::Nul)},
            Token::ListT => {
                self.next()?;
                let list_typ = self.parse_type()?;
                UType::List(Box::new(list_typ))
            },
            Token::UnknownT => {
                self.next()?;
                UType::Resolved(Type::Unknown)
            },
            Token::Scope => {
                self.next()?;
                UType::InferScope
            }

            other => unreachable!("{other:?}")
        });
    }

    pub fn parse_type(&mut self) -> Res<UType> {
        let mut ty = self.parse_type_postfix()?;
    
        loop {
            match self.peek()?.token() {
                Token::DoubleOr => {
                    self.next()?;
                    let rhs = self.parse_type_postfix()?;
                    ty = UType::Union(Box::new(ty), Box::new(rhs));
                }
    
                Token::Or => {
                    self.next()?;
                    let rhs = self.parse_type_postfix()?;
    
                    match ty {
                        UType::SafeUnion(mut types) => {
                            types.push(rhs);
                            ty = UType::SafeUnion(types);
                        }
    
                        other => {
                            ty = UType::SafeUnion(vec![other, rhs]);
                        }
                    }
                }
    
                _ => break,
            }
        }
    
        Ok(ty)
    }
    
    fn parse_type_postfix(&mut self) -> Res<UType> {
        let span = self.expect_peek(exp_type_starter())?;
        let (_, t) = span.destructure();
    
        let mut ty = self.parse_type_atom(t)?;
    
        loop {
            match self.peek()?.token() {
                Token::Dot => {
                    self.next()?;
    
                    let initial_span = self.expect(exp_ident())?;
                    let (_, Token::Ident(id)) = initial_span.destructure()
                        else { unreachable!() };
    
                    ty = UType::Property(Box::new(ty), id);
                }
    
                Token::LBracket => {
                    let x = self.parse_nominal_application()?;
                    ty = UType::Apply(Box::new(ty), x);
                }
    
                _ => break,
            }
        }
    
        Ok(ty)
    }
}

pub fn exp_named_arg_member() -> TokenExpectation {
    tComb!(
        "Ident | Def | RParen",
        exp_ident(),
        tExp!(RParen, Def)
    )
}

pub fn exp_type_starter() -> TokenExpectation {
    tComb!(
        "Fn | Atsign | IntT | FloatT | StrT | CharT | BoolT | ListT | AddrT | UnknownT | AnyT | Undefined | Ident | Nul | Scope",
        tExp!(Fn, Atsign, IntT, FloatT, StrT, CharT, BoolT, ListT, AddrT, UnknownT, Undefined, AnyT, LParen, Nul, Scope),
        exp_ident(),
    )
}