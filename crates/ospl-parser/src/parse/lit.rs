use ospl_common::ast::{ArgNaming, FunctionType, FunctionValue, Scope, Type, UType};
use crate::{lexer::token::{exp_ident, Token, TokenExpectation}, parse::{Parser, Res}, tComb, tExp};

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
        let mut arg_values = Vec::new();
        self.expect(tExp!(LParen))?;
        loop {
            let (_, token) = self.expect(exp_named_arg_member())?.destructure();
            match token {
                Token::Ident(i) => arg_values.push(ArgNaming {
                    name: i,
                }),
                Token::Def => {
                    let span = self.expect(exp_ident())?;
                    let (_, Token::Ident(t)) = span.destructure()
                        else { unreachable!() };
                    
                    arg_values.push(ArgNaming {
                        name: t,
                    });
                }
                Token::RParen => break,
                _ => unreachable!()
            }

            self.expect(tExp!(Colon))?;

            arg_types.push(self.parse_type()?);
        }  // NOTE: we consumed RParen in the loop

        let ret = self.parse_function_return_type()?;

        let b = self.parse_block()?;
        return Ok(FunctionValue {
            ftype: FunctionType {
                args: arg_types,
                ret
            },
            block: b,
            args: arg_values,
            captures,
        })
    }

    pub fn parse_function_generics(&mut self) -> Res<Vec<(String, UType)>> {
        let mut types = Vec::new();
        loop {
            let span = self.expect_peek(tComb!(
                "RAngle | start of ID",
                tExp!(RAngle),
                exp_ident(),
            ))?;

            if let Token::RAngle = span.token() {
                self.next()?;  // consume `>`
                break;
            }

            if let (_, Token::Ident(i)) = span.destructure() {
                self.next()?;
                self.expect(tExp!(Colon))?;
                let ty = self.parse_type()?;
                types.push((i, ty));
            }
        };

        return Ok(types)
    }

    pub fn parse_function_type(&mut self) -> Res<FunctionType<UType>> {
        self.expect(tExp!(Fn))?;

        // arg types
        self.expect(tExp!(LParen))?;
        let mut args = Vec::new();
        loop {
            let s = self.expect_peek(exp_arg_member())?;
            if *s.token() == Token::RParen {
                break;
            }
            
            // if it's not the end of the list
            let t = self.parse_type()?;
            args.push(t);
        }
        self.expect(tExp!(RParen))?;

        let ret = self.parse_function_return_type()?;

        return Ok(FunctionType {
            args,
            ret,
        });
    }

    pub fn parse_function_return_type(&mut self) -> Res<UType> {
        // get return type (defaults to nul)
        let ret = if let Token::Arrow = self.peek()?.token() {
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
            Token::ListT => {
                self.next()?;
                let list_typ = self.parse_type()?;
                UType::List(Box::new(list_typ))
            },
            Token::UnknownT => {
                self.next()?;
                UType::Resolved(Type::Unknown)
            },

            Token::Scope => self.parse_scope_type()?,
            other => unreachable!("{other:?}")
        });
    }

    pub fn parse_type(&mut self) -> Res<UType> {
        let span = self.expect_peek(exp_type_starter())?;
        let (_, t) = span.destructure();

        let mut working_type = self.parse_type_atom(t)?;

        loop {
            if let Token::Dot = self.peek()?.token() {
                self.next()?;

                let initial_span = self.expect(exp_ident())?;
                let (_, Token::Ident(id)) = initial_span.destructure()
                    else { unreachable!() };

                working_type = UType::Property(Box::new(working_type), id);
            }
            else if let Token::LogicOr = self.peek()?.token() {
                self.next()?;
                let x = self.parse_type()?;
                working_type = UType::Union(Box::new(working_type), Box::new(x));
            }
            else if let Token::LBracket = self.peek()?.token() {
                let x = self.parse_nominal_application()?;

                working_type = UType::Apply(Box::new(working_type), x);
            }
            else { break; }
        }

        return Ok(working_type)
    }

    pub fn parse_scope_type(&mut self) -> Res<UType> {
        self.expect(tExp!(Scope))?;

        if *self.peek()?.token() != Token::LParen {
            return Ok(UType::InferScope)
        }

        self.expect(tExp!(LParen))?;
        let mut scope: Scope<UType> = Scope::default();
        let mut current = 0;  // imitate addresses being correct
        loop {
            let (_, token) = self.expect(tComb!(
                "Ident | RParen",
                exp_ident(),
                tExp!(RParen),
            ))?.destructure();
            let name = match token {
                Token::Ident(i) => i,
                Token::RParen => break,
                _ => unreachable!()
            };

            self.expect(tExp!(Colon))?;

            let ty = self.parse_type()?;
            scope.declare(name, current, ty);

            current += 1;
        }  // NOTE: we consumed RParen in the loop

        return Ok(UType::Scope(scope))
    }
}

pub fn exp_arg_member() -> TokenExpectation {
    tComb!(
        "EXP_TYPE_STARTER | RParen",
        exp_type_starter(),
        tExp!(RParen)
    )
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
        "Fn | Atsign | IntT | FloatT | StrT | CharT | BoolT | ListT | AddrT | UnknownT | AnyT | Undefined | Ident | Scope",
        tExp!(Fn, Atsign, IntT, FloatT, StrT, CharT, BoolT, ListT, AddrT, UnknownT, Undefined, AnyT, LParen, Scope),
        exp_ident(),
    )
}