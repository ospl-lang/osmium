use ospl_common::ast::Scope;

use crate::{Compiler, RelativeVarID, ScopeStack};

impl Compiler {
    /// Increments to the next index, returning the previous one,
    /// like a postincrement
    fn next_var(&mut self) -> RelativeVarID {
        let t = self.stack.top_mut();
        return t.next_post()
    }

    pub fn new() -> Self {
        return Self {
            stack: ScopeStack { scopes: vec![
                Scope::default()
            ] }
        }
    }
}

impl ScopeStack {
    pub fn top(&self) -> &Scope {
        return self.scopes.last().unwrap()
    }

    pub fn top_mut(&mut self) -> &mut Scope {
        return self.scopes.last_mut().unwrap()
    }
}

mod stmt;
mod expr;
mod func;
mod binaryop;
