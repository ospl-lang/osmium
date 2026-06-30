use std::collections::HashMap;
use tracing::instrument;

use crate::{lexer::token::{Span, TokenExpectation}, parse::{macro_interpreter::MacroValue, macros::Macro}};

#[derive(Debug)]
pub enum PE {
    Expected(TokenExpected),
    RequiredPrimitiveType,
    UnexpectedEOF,
    EOF,
    MacroCallWithNoMacrosToCall,
    NoSuchMacro(String),
    NoSuchMacroVariable(String),
    MacroInvocationError(Box<PE>),
    MacroParsingError(Box<PE>),
    UnsupportedMacroOperation(MacroValue, MacroValue),
    InnapropriateMacroForPlace {
        mac: Macro,
        placename: String
    }
}

pub type Res<T> = Result<T, PE>;

#[derive(Debug)]
pub struct TokenExpected {
    pub expected: String,
    pub got: Span,
}

#[derive(Clone, Debug)]
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

    #[instrument]
    fn expect(&mut self, exp: TokenExpectation) -> Res<Span> {
        let got = self.next()?;
        if (exp.matches)(got.token()) {
            tracing::trace!("Unexpected token in expect()");
            return Ok(got)
        }

        return Err(PE::Expected(TokenExpected {
            expected: exp.label.to_string(),
            got: got.clone(),
        }))
    }

    #[instrument]
    fn expect_peek(&mut self, exp: TokenExpectation) -> Res<Span> {
        let got = self.peek()?;
        if (exp.matches)(got.token()) {
            tracing::trace!("Unexpected token in expect_peek()");
            return Ok(got)
        }

        return Err(PE::Expected(TokenExpected {
            expected: exp.label.to_string(),
            got: got.clone(),
        }))
    }

    #[instrument]
    fn next(&mut self) -> Res<Span> {
        let ct = self.current_token;
        let Some(t) = self.tokens.get(ct)
        else {
            tracing::trace!("Unexpected EOF in next()");
            return Err(PE::UnexpectedEOF)
        };

        self.current_token += 1;

        return Ok(t.clone())
    }

    #[instrument]
    fn peek(&self) -> Res<Span> {
        let Some(t) = self.tokens.get(self.current_token)  
        else {
            tracing::trace!("Unexpected EOF in peek()");
            return Err(PE::EOF)
        };

        return Ok(t.clone())
    }
}

mod stmt;
mod expr;
mod lit;
mod cond;
mod ffi;
pub mod macros;
pub mod macro_interpreter;

pub mod diag;

#[cfg(test)]
mod tests;