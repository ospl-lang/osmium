use std::sync::Arc;

use ospl_common::ast::{Scope, Type};

use crate::{BuildData, Compiler, RelativeVarID, ScopeStack};

impl Compiler {
    /// Increments to the next index, returning the previous one,
    /// like a postincrement
    fn next_var(&mut self) -> RelativeVarID {
        let t = self.stack.top_mut();
        return t.next_post()
    }

    pub fn new(bd: Arc<BuildData>) -> Self {
        return Self {
            stack: ScopeStack { scopes: vec![
                Scope::default()
            ] },
            bd
        }
    }
}

impl ScopeStack {
    pub fn top(&self) -> &Scope<Type> {
        return self.scopes.last().unwrap()
    }

    pub fn top_mut(&mut self) -> &mut Scope<Type> {
        return self.scopes.last_mut().unwrap()
    }

    #[allow(unused)]
    pub fn push(&mut self) {
        self.scopes.push(Scope::default());
    }

    pub fn push_parental(&mut self) {
        let s = self.top().clone();
        self.scopes.push(s);
    }

    pub fn pop(&mut self) {
        self.scopes.pop();
    }
}

mod stmt;
mod expr;
mod func;
mod cond;
mod array;
mod binaryop;
mod unaryop;
mod ffi;
mod types;