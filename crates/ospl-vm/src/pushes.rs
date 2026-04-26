use ospl_common::inst::RuntimeStaticValue;

use crate::{VM, Value, arena::ArenaIndex};

impl VM {
    pub fn push_copy(&mut self, v: &RuntimeStaticValue) -> ArenaIndex {
        let i = self.arena.push(v.clone_composite());
        self.top_mut().indexes.push(i);
        return i
    }
}