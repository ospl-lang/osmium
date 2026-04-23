use crate::{VM, arena::ArenaIndex};

impl VM {
    pub fn builtin_print(&mut self, arg: ArenaIndex) {
        print!("{:?}", self.arena.get(arg))
    }
}
