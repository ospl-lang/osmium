//! TODO: ADD CYCLE DETECTION.

use std::collections::HashSet;

use ospl_common::inst::RuntimeValue;

use crate::{VM, arena::MEMMAX};

impl VM {
    fn trace_value(&self, value: &RuntimeValue, out: &mut HashSet<usize>) {
        match value {
            RuntimeValue::Function(f) => out.extend(&f.captures),
            RuntimeValue::List(l) => out.extend(&l.items),
            RuntimeValue::Scope(s) => out.extend(&s.indexes),
            _ => {}
        }
    }

    pub fn gc(&mut self) {
        let mut marked = HashSet::with_capacity(MEMMAX);
        for root in self.stack.iter() {
            for index in root.indexes.iter() {
                let value = self.arena.get(*index);
                self.trace_value(value, &mut marked);
            }
        }

        // O(n + k)
        for possibly in 0..MEMMAX {
            if marked.contains(&possibly) {
                // live to see another day!
                continue;
            } else {
                // we're DEAD
                self.arena.reclaim(possibly);
            }
        }
    }
}
