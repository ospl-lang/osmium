use crate::{VM, Value, arena::ArenaIndex};

impl VM {
    pub fn push_copy(&mut self, v: &Value) -> ArenaIndex {
        let i = self.arena.push(v.clone_composite());
        self.top_mut().indexes.push(i);
        return i
    }
}