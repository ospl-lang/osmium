use crate::{Keyword, Parser};
use ::ospl_common::ast::repr::{Expr, LValue};

impl<'a> Parser<'a> {
    const B_OPEN_GROUP: char = '[';
    const B_CLOSE_GROUP: char = ']';

    pub fn parse_ident(&mut self) -> Option<String> {
        // ensure our first character is alphabetic
        let mut output = String::new();
        if let Some(ch) = self.peek() {
            if !ch.is_alphabetic() {
                return None
            }

            output.push(ch);
            self.pos += ch.len_utf8();
        }

        let the_rest_of_the_name = self.read_until(|ch| !ch.is_alphanumeric());
        output.push_str(the_rest_of_the_name);

        if Keyword::str_is_keyword(&output) {
            return None
        }

        return Some(output)
    }

    pub fn parse_lvalue(&mut self) -> Option<LValue> {
        // currently only variables are supported
        if let Some(id) = self.parse_ident() {
            // TODO: can handle stuff here
            return Some(LValue::Var(id))
        }

        return None
    }

    pub fn parse_atom(&mut self) -> Option<Expr> {
        if let Some(v) = self.parse_literal() {
            return Some(Expr::StaticLiteral(v))
        }

        if let Some(Self::B_OPEN_GROUP) = self.peek() {
            let r = self.parse_expr()?;

            let Some(Self::B_CLOSE_GROUP) = self.peek()
                else { return None };

            return Some(r)
        }

        return None
    }

    pub fn parse_expr(&mut self) -> Option<Expr> {
        // do parsing here
        return self.parse_atom()
    }
}
