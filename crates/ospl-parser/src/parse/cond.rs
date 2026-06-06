use ospl_common::ast::{Statement, Stmt};

use crate::{lexer::token::{Token, TokenExpectation}, parse::{Parser, Res}, tExp};

impl<'a> Parser<'a> {
    pub fn parse_if(&mut self) -> Res<Statement> {
        let start = self.expect(tExp!(If))?;

        let left = self.parse_expr()?;
        let yes = self.parse_block()?;

        let no = if let Token::Else = self.peek()?.token() {
            self.next()?;
            self.parse_block()?
        } else { Vec::new() };

        return Ok(Statement {
            at: *start.position(),
            inner: Box::new(Stmt::If(
                left,
                yes,
                no
            ))
        })
    }

    pub fn parse_loop(&mut self) -> Res<Statement> {
        let start = self.expect(tExp!(Loop))?;
        let x = self.parse_block()?;
        return Ok(Statement {
            at: *start.position(),
            inner: Box::new(Stmt::Loop(x)),
        })
    }
}