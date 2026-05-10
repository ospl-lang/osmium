use std::{any::Any, fmt::Debug};

use crate::ast::{Expression, LValue, Position, Statement};

pub trait Spannable: Any + Debug {
    fn get_pos(&mut self) -> Position;

    // ugly hack to get around some crap
    fn spanned(&self) -> Box<dyn Spannable>;
}

impl Spannable for Statement {
    fn get_pos(&mut self) -> Position {
        self.at
    }

    fn spanned(&self) -> Box<dyn Spannable> {
        return Box::new(self.clone())
    }
}

impl Spannable for Expression {
    fn get_pos(&mut self) -> Position {
        self.at
    }

    fn spanned(&self) -> Box<dyn Spannable> {
        return Box::new(self.clone())
    }
}

impl Spannable for LValue {
    fn get_pos(&mut self) -> Position {
        self.at
    }

    fn spanned(&self) -> Box<dyn Spannable> {
        return Box::new(self.clone())
    }
}
