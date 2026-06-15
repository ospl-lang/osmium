use std::hint::unreachable_unchecked;

use ospl_common::inst::{RT, list::List, make};

use crate::VM;

impl VM {
    pub fn dec_value(&mut self, i: usize) {
        let t = self.get_mut_value_top(i);
        unsafe { match t.tag {
            RT::List => {
                let x: &mut List = &mut t.data.list;
                let Some(i) = x.items.pop()
                else {
                    self.push_literal(make::undefined(()));
                    return;
                };

                self.stack.top_add_index(i);
            },
            RT::Scope => {
                let l = make::list(List {
                    items: t.data.list.items.clone()
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