use crate::Parser;
use ::ospl_common::ast::repr::StaticValue;

impl<'a> Parser<'a> {
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

        return None
    }
}