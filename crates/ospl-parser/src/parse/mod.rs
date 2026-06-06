use crate::lexer::token::{Span, TokenExpectation};

#[derive(Debug)]
pub enum PE {
    Expected(TokenExpected),
    RequiredPrimitiveType,
    UnexpectedEOF,
}

pub type Res<T> = Result<T, PE>;

#[derive(Debug)]
pub struct TokenExpected {
    pub expected: String,
    pub got: Span,
}

pub struct Parser<'a> {
    tokens: &'a [Span],
    current_token: usize,
}

impl<'a> Parser<'a> {
    pub fn new(tokens: &'a [Span]) -> Self {
        return Self {
            tokens,
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
}

mod stmt;
mod expr;
mod lit;
mod cond;
mod ffi;

pub mod diag;

#[cfg(test)]
mod tests;