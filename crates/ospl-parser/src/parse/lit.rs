use ospl_common::ast::{FunctionType, FunctionValue, Scope, Type};
use crate::{lexer::token::{EXP_IDENT, Token, TokenExpectation}, parse::{Parser, Res}, tComb, tExp};

impl<'a> Parser<'a> {
    pub fn parse_function_literal(&mut self) -> Res<FunctionValue> {
        // two paralel lists: very fucking stupid
        let mut arg_types = Vec::new();
        let mut arg_names = Vec::new();

        self.expect(tExp!(Fn))?;
        self.expect(tExp!(LParen))?;
        loop {
            let (_, token) = self.expect(EXP_NAMED_ARG_MEMBER)?.destructure();
            match token {
                Token::Ident(i) => arg_names.push(i),
                Token::RParen => break,
                _ => unreachable!()
            }

            self.expect(tExp!(Colon))?;

            arg_types.push(self.parse_type()?);
        }  // NOTE: we consumed RParen in the loop

        let ret = self.parse_function_return_type()?;

        let b = self.parse_block()?;
        return Ok(FunctionValue {
            ftype: FunctionType { args: arg_types, ret },
            block: b,
            args: arg_names,
        })
    }

    pub fn parse_function_type(&mut self) -> Res<FunctionType> {
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
            ret
        });
    }

    pub fn parse_function_return_type(&mut self) -> Res<Type> {
        // get return type (defaults to nul)
        let ret = if let Token::Arrow = self.peek()?.token() {
            self.next()?;  // take arrow
            self.parse_type()?
        } else {
            Type::Nul
        };

        return Ok(ret)
    }

    pub fn parse_type(&mut self) -> Res<Type> {
        let span = self.expect_peek(EXP_TYPE_STARTER)?;
        let (_, t) = span.destructure();

        match t {
            Token::Fn => Ok(Type::Function(Box::new(self.parse_function_type()?))),
            Token::Ident(i) => {self.next()?; return Ok(Type::TypeOfVar(i))},

            /* primitives */
            Token::IntT => {self.next()?; return Ok(Type::Int)},
            Token::FloatT => {self.next()?; return Ok(Type::Float)},
            Token::StrT => {self.next()?; return Ok(Type::Str)},
            Token::BoolT => {self.next()?; return Ok(Type::Bool)},
            Token::ListT => {self.next()?; return Ok(Type::List)},

            Token::Atsign => unimplemented!(),
            Token::Scope => return Ok(Type::Scope(self.parse_scope_type()?)),
            other => unreachable!("{other:?}")
        }
    }

    pub fn parse_scope_type(&mut self) -> Res<Scope> {
        self.expect(tExp!(Scope))?;
        self.expect(tExp!(LSquirly))?;
        let mut scope = Scope::default();
        let mut current = 0;  // imitate addresses being correct
        loop {
            let (_, token) = self.expect(tComb!(
                "Ident | RSquirly",
                EXP_IDENT,
                tExp!(RSquirly),
            ))?.destructure();
            let name = match token {
                Token::Ident(i) => i,
                Token::RSquirly => break,
                _ => unreachable!()
            };

            self.expect(tExp!(Colon))?;

            let ty = self.parse_type()?;
            scope.declare(name, current, ty);

            current += 1;
        }  // NOTE: we consumed RParen in the loop

        return Ok(scope)
    }
}

const EXP_ARG_MEMBER: TokenExpectation = tComb!(
    "EXP_TYPE_STARTER | RParen",
    EXP_TYPE_STARTER,
    tExp!(RParen)
);

const EXP_NAMED_ARG_MEMBER: TokenExpectation = tComb!(
    "Ident | RParen",
    EXP_IDENT,
    tExp!(RParen)
);

pub const EXP_TYPE_STARTER: TokenExpectation = tComb!(
    "Fn | Atsign | IntT | FloatT | StrT | BoolT | ListT | Ident | Scope",
    tExp!(Fn, Atsign, IntT, FloatT, StrT, BoolT, ListT, Scope),
    EXP_IDENT,
);
