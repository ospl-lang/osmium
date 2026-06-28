use std::collections::HashMap;

use crate::lexer::token::{Span, TokenExpectation};

#[derive(Debug)]
pub enum PE {
    Expected(TokenExpected),
    RequiredPrimitiveType,
    UnexpectedEOF,
    MacroCallWithNoMacrosToCall,
    NoSuchMacro(String),
    NoSuchMacroInput(String),
    NoSuchMacroConstruct(String),
    MacroInvocationError(Box<PE>)
}

pub type Res<T> = Result<T, PE>;

#[derive(Debug)]
pub struct TokenExpected {
    pub expected: String,
    pub got: Span,
}

#[derive(Clone)]
pub struct Parser<'a> {
    tokens: &'a [Span],
    local_macros: Vec<HashMap<String, macros::Macro>>,
    current_token: usize,
}

impl<'a> Parser<'a> {
    pub fn new(tokens: &'a [Span]) -> Self {
        return Self {
            tokens,
            local_macros: Vec::new(),
            current_token: 0,
        }
    }

    fn expect(&mut self, exp: TokenExpectation) -> Res<Span> {
        let got = self.next()?;
        if (exp.matches)(got.token()) {
            return Ok(got)
        }

        return Err(PE::Expected(TokenExpected {
            expected: exp.label.to_string(),
            got: got.clone(),
        }))
    }

    fn expect_peek(&mut self, exp: TokenExpectation) -> Res<Span> {
        let got = self.peek()?;
        if (exp.matches)(got.token()) {
            return Ok(got)
        }

        return Err(PE::Expected(TokenExpected {
            expected: exp.label.to_string(),
            got: got.clone(),
        }))
    }

    fn next(&mut self) -> Res<Span> {
        let ct = self.current_token;
        let Some(t) = self.tokens.get(ct)
        else {
            // panic!("unexpected EOF");
            return Err(PE::UnexpectedEOF)
        };

        self.current_token += 1;

        return Ok(t.clone())
    }

    fn peek(&self) -> Res<Span> {
        let Some(t) = self.tokens.get(self.current_token)  
        else {
            // panic!("unexpected EOF in peek()");
            return Err(PE::UnexpectedEOF)
        };

        return Ok(t.clone())
    }

    fn peekn(&self, next_n: usize) -> Res<Span> {
        let Some(t) = self.tokens.get(self.current_token + next_n)  
        else {
            // panic!("unexpected EOF in peek()");
            return Err(PE::UnexpectedEOF)
        };

        return Ok(t.clone())
    }
}

mod stmt;
mod expr;
mod lit;
mod cond;
mod ffi;
mod macros;

pub mod diag;

#[cfg(test)]
mod tests;