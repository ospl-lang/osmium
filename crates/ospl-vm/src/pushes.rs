use ospl_common::inst::RuntimeValue;

use crate::{VM, arena::ArenaIndex};

impl VM {
    pub fn push_copy(&mut self, v: &RuntimeValue) -> ArenaIndex {
        let i = self.arena.push(v.clone());
        self.top_mut().indexes.push(i);
        return i
    }
}
