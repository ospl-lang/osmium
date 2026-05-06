use std::{any::Any, fmt::Debug};

use crate::ast::{Expression, LValue, Position, Statement};

pub trait Spannable: Any + Debug {
    fn get_pos(&mut self) -> Position;
}

impl Spannable for Statement {
    fn get_pos(&mut self) -> Position {
        self.at
    }
}

impl Spannable for Expression {
    fn get_pos(&mut self) -> Position {
        self.at
    }
}

impl Spannable for LValue {
    fn get_pos(&mut self) -> Position {
        self.at
    }
}

#[derive(Debug)]
pub struct IdkWhere;

impl Spannable for IdkWhere {
    fn get_pos(&mut self) -> Position {
        // TODO: warn
        return Position {
            line: usize::MAX,
            column: usize::MAX,
            ch: usize::MAX,
            token_num: usize::MAX
        }
    }
}