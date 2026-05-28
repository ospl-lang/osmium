use ospl_common::ast::{ArgNaming, FunctionType, FunctionValue, Scope, Type, UType, decl::Visibility};
use crate::{lexer::token::{EXP_IDENT, Token, TokenExpectation}, parse::{Parser, Res}, tComb, tExp};

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
                    EXP_IDENT
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
            let (_, token) = self.expect(EXP_NAMED_ARG_MEMBER)?.destructure();
            match token {
                Token::Ident(i) => arg_values.push(ArgNaming {
                    name: i,
                    privacy: Visibility::Public,
                }),
                Token::Def => {
                    let span = self.expect(EXP_IDENT)?;
                    let (_, Token::Ident(t)) = span.destructure()
                        else { unreachable!() };
                    
                    arg_values.push(ArgNaming {
                        name: t,
                        privacy: Visibility::Public
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
                EXP_IDENT,
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
            let s = self.expect_peek(EXP_ARG_MEMBER)?;
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

    pub fn parse_type(&mut self) -> Res<UType> {
        let span = self.expect_peek(EXP_TYPE_STARTER)?;
        let (_, t) = span.destructure();

        match t {
            Token::Fn => Ok(UType::Function(Box::new(self.parse_function_type()?))),
            Token::Ident(i) => {self.next()?; return Ok(UType::Typeof(i))},
            Token::Atsign => {
                self.next()?;
                let t = self.parse_type()?;
                return Ok(UType::Returnof(Box::new(t)))
            }

            /* primitives */
            Token::IntT => {self.next()?; return Ok(UType::Resolved(Type::Int))},
            Token::FloatT => {self.next()?; return Ok(UType::Resolved(Type::Float))},
            Token::StrT => {self.next()?; return Ok(UType::Resolved(Type::Str))},
            Token::CharT => {self.next()?; return Ok(UType::Resolved(Type::Char))},
            Token::BoolT => {self.next()?; return Ok(UType::Resolved(Type::Bool))},
            Token::AddrT => {self.next()?; return Ok(UType::Resolved(Type::Address))},
            Token::ListT => {
                self.next()?;
                let list_typ = self.parse_type()?;
                return Ok(UType::List(Box::new(list_typ)))
            },
            Token::UnknownT => {
                self.next()?;
                return Ok(UType::Resolved(Type::Unknown))
            }

            Token::Scope => return Ok(self.parse_scope_type()?),
            other => unreachable!("{other:?}")
        }
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
                "Ident | RParen | Semicolon",
                EXP_IDENT,
                tExp!(RParen, Semicolon),
            ))?.destructure();
            let name = match token {
                Token::Ident(i) => i,
                Token::RParen => break,
                Token::Semicolon => continue,
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

const EXP_ARG_MEMBER: TokenExpectation = tComb!(
    "EXP_TYPE_STARTER | RParen",
    EXP_TYPE_STARTER,
    tExp!(RParen)
);

const EXP_NAMED_ARG_MEMBER: TokenExpectation = tComb!(
    "Ident | Def | RParen",
    EXP_IDENT,
    tExp!(RParen, Def)
);

pub const EXP_TYPE_STARTER: TokenExpectation = tComb!(
    "Fn | Atsign | IntT | FloatT | StrT | CharT | BoolT | ListT | AddrT | UnknownT | Ident | Scope",
    tExp!(Fn, Atsign, IntT, FloatT, StrT, CharT, BoolT, ListT, AddrT, UnknownT, Scope),
    EXP_IDENT,
);
