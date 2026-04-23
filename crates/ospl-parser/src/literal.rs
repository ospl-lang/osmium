use crate::{Keyword, Parser};
use ospl_common::ast::repr::FunctionData;
use ::ospl_common::ast::repr::StaticValue;

impl<'a> Parser<'a> {
    pub fn function_literal(&mut self) -> Option<FunctionData> {
        self.keywords_comsume(&[Keyword::Fn])?;
        self.skip_ws();

        let ty = self.parse_function_type()?;
        self.skip_ws();
        let code = self.parse_block()?;

        return Some(FunctionData { code, ty })
    }

    pub fn string_literal(&mut self) -> Option<String> {
        self.expect_char('"')?;

        let s = self.read_until(|e| e == '"')
            .to_string();  // maybe don't?

        self.expect_char('"')?;  // maybe remove?

        return Some(s)
    }

    pub fn whole_number_literal(&mut self) -> Option<i64> {
        let mut s = String::new();

        if let Some(ch) = self.peek()
            && (ch == '+' || ch == '-') {
            self.pos += ch.len_utf8();
            s.push(ch);
        }

        s.push_str(self.read_until(|ch| !ch.is_numeric()));

        if s.is_empty() || s == "+" || s == "-" {
            return None
        } else {
            return s.parse::<i64>().ok()
        }
    }

    pub fn parse_literal(&mut self) -> Option<StaticValue> {
        if let Some(n) = self.whole_number_literal() {
            return Some(StaticValue::Int(n))
        }

        else if let Some(s) = self.string_literal() {
            return Some(StaticValue::Str(s))
        }

        else if let Some(f) = self.function_literal() {
            return Some(StaticValue::Function(f))
        }

        return None
    }
}