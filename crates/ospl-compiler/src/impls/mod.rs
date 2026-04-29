use crate::{Compiler, RelativeVarID, Scope, ScopeStack, Type};

impl Compiler {
    /// Increments to the next index, returning the previous one,
    /// like a postincrement
    pub fn next_var(&mut self) -> RelativeVarID {
        let t = self.stack.top_mut();
        let i = t.next_id;
        t.next_id += 1;
        return i
    }
}

impl ScopeStack {
    pub fn top(&self) -> &super::Scope {
        return self.scopes.last().unwrap()
    }

    pub fn top_mut(&mut self) -> &mut super::Scope {
        return self.scopes.last_mut().unwrap()
    }
}

impl Scope {
    pub fn declare(&mut self, key: String, address: usize, ty: Type) {
        self.map.insert(key, crate::Store {
            ty,
            var: address
        });
    }
}

impl PartialEq for Scope {
    fn eq(&self, other: &Self) -> bool {
        return self.map == other.map
    }
}

mod stmt;
mod expr;
mod func;
