use std::sync::Arc;

use tracing::instrument;

use crate::{lexer::token::{Span, Token, TokenExpectation, exp_ident}};

#[derive(Debug)]
pub enum PE {
    Expected(TokenExpected),
    EOF,
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
    current_token: usize,
    filename: Arc<String>
}

impl<'a> Parser<'a> {
    pub fn new(tokens: &'a [Span], file: String) -> Self {
        return Self {
            tokens,
            current_token: 0,
            filename: Arc::new(file)
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
            return Err(PE::EOF)
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

    pub fn parse_ident(&mut self) -> Res<String> {
        let id = self.expect(exp_ident())?;
        let (_, Token::Ident(id)) = id.destructure()
        else { unreachable!() };

        return Ok(id)
    }
}

mod stmt;
mod expr;
mod lit;
mod cond;
mod ffi;

pub mod diag;

#[cfg(test)]
mod tests;