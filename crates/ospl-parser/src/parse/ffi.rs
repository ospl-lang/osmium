use std::sync::Arc;

use ospl_common::ast::{Expr, Expression};

use crate::{lexer::token::{exp_ident, Token, TokenExpectation}, parse::{Parser, Res}, tComb, tExp};

pub fn exp_int() -> TokenExpectation {
    TokenExpectation {
        matches: Box::new(|t| matches!(t, Token::Integer(_))),
        label: "Integer"
    }
}

#[allow(unused)]
pub fn exp_addr() -> TokenExpectation {
    TokenExpectation {
        matches: Box::new(|t| matches!(t, Token::AddressLiteral(_))),
        label: "AddressLiteral"
    }
}

impl<'a> Parser<'a> {
    pub fn parse_foreign_expr(&mut self) -> Res<Expression> {
        self.expect(tExp!(Foreign))?;

        let span = self.expect_peek(tComb!(
            "Fn | Use | begining of lvalue",
            tExp!(Fn),
            tExp!(Use),
            exp_ident(),
        ))?;
        match span.token() {
            Token::Use => self.parse_foreign_load(),
            Token::Fn => self.parse_foreign_fn(),
            Token::Ident(_) => self.parse_foreign_call(),
            _ => unreachable!()
        }
    }

    fn parse_foreign_fn(&mut self) -> Res<Expression> {
        self.expect(tExp!(Fn))?;

        let libname = self.parse_lvalue()?;
        let fname = self.parse_atom()?;

        let id = self.expect(exp_ident())?;
        let Token::Ident(id) = id.token() else { unreachable!() };

        let rtype = id_to_type_number(id);

        let mut type_numbers = Vec::new();
        self.expect(tExp!(LSquirly))?;
        loop {
            let span = self.expect(tComb!(
                "RSquirly | Ident | Integer",
                tExp!(RSquirly),
                exp_ident(),
                exp_int(),
            ))?;

            match span.token() {
                Token::RSquirly => break,
                Token::Ident(i) => type_numbers.push(id_to_type_number(i)),
                Token::Integer(i) => type_numbers.push(*i as usize),
                _ => unreachable!()
            }
        }  // consumed `}` in the loop

        return Ok(Expression {
            at: libname.at,
            inner: Box::new(Expr::FFIFunc(
                libname,
                fname,
                rtype,
                type_numbers
            )),
            file: Arc::clone(&self.filename),
        })
    }

    fn parse_foreign_load(&mut self) -> Res<Expression> {
        self.expect(tExp!(Use))?;

        let a = self.parse_atom()?;

        return Ok(Expression {
            at: a.at,
            inner: Box::new(Expr::FFILoad(a)),
            file: Arc::clone(&self.filename),
        })
    }

    fn parse_foreign_call(&mut self) -> Res<Expression> {
        let left = self.parse_lvalue()?;

        // TODO - throw error if there are named args
        let (args, _) = self.parse_fn_call_args()?;

        return Ok(Expression {
            at: left.at,
            inner: Box::new(Expr::FFICall(left, args)),
            file: Arc::clone(&self.filename),
        })
    }
}

fn id_to_type_number(i: &str) -> usize {
    return match i {
        "u8" => 0,
        "i8" => 1,
        "u16" => 2,
        "i16" => 3,
        "u32" => 4,
        "i32" => 5,
        "u64" => 6,
        "i64" => 7,
        "f32" => 8,
        "f64" => 9,
        "void" => 10,
        "ptr" => 11,
        other => unimplemented!("unknown FFI type: {other}")  // TODO - error here
    }
}