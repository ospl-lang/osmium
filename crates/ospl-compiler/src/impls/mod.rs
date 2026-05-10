use ospl_common::ast::Scope;

use crate::{Compiler, RelativeVarID, ScopeStack, package::Packages};

impl Compiler {
    /// Increments to the next index, returning the previous one,
    /// like a postincrement
    fn next_var(&mut self) -> RelativeVarID {
        let t = self.stack.top_mut();
        return t.next_post()
    }

    pub fn new(pkgs: Packages) -> Self {
        return Self {
            stack: ScopeStack { scopes: vec![
                Scope::default()
            ] },
            packages: pkgs
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
mod pkg;
mod types;