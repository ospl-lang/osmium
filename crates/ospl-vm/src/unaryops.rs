use std::hint::unreachable_unchecked;

use ospl_common::inst::{RuntimeValue, list::List};

use crate::VM;

impl VM {
    pub fn dec_value(&mut self, i: usize) {
        let t = self.get_mut_value_top(i);
        unsafe { match t {
            RuntimeValue::List(l) => {
                let i = l.items.pop().expect("failed");
                self.stack.top_add_index(i);
            },
            RuntimeValue::Scope(s) => {
                let l = RuntimeValue::List(List {
                    items: s.indexes.clone()
                });

                self.push_literal(l);
            }
            _ => unreachable_unchecked()
        } };
    }

    pub fn copyof(&mut self, i: usize) {
        let t = self.get_value_top(i);
        let t = t.clone();
        self.push_literal(t);
    }
}