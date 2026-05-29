use ospl_common::inst::{RuntimeValue, list::List};

use crate::VM;

impl VM {
    pub fn push_array(&mut self, indexes: &[usize]) {
        let mut list = List::default();
        for rel in indexes {
            let abs = self.top().indexes[*rel];
            self.arena.inc_refcount(abs);
            
            list.items.push(abs);
        }

        self.push_literal(RuntimeValue::List(list));
    }

    pub fn index(&mut self, array: usize, index: usize) {
        // SAFETY: the invariant here is that the instruction is operating on a
        // valid type and the index into the array is valid.
        let x = {
            let indexable = self.get_value_top(array);
            let x = match indexable {
                RuntimeValue::List(l) => {
                    let index = unsafe { self.get_value_top(index).assume_int() };
                    l.items[index as usize]
                },
                RuntimeValue::Map(m) => {
                    let key = self.get_value_top(index).as_str().expect("map indexed by not a string");
                    *m.get(key).expect("failed to index map: key not found?")
                },
                RuntimeValue::Str(s) => {
                    let index = unsafe { self.get_value_top(index).assume_int() };
                    let Some(x) = s.chars().nth(index as usize)
                    else {
                        self.push_literal(RuntimeValue::Undefined);  // out of bounds
                        return;
                    };

                    self.push_literal(RuntimeValue::Char(x));
                    return;
                },
                _ => unsafe{ std::hint::unreachable_unchecked() },
            };

            x
        };
        self.top_mut().indexes.push(x);
    }

    pub fn slice(&mut self, array: usize, start: usize, end: usize) {
        let x = unsafe {
            let list = self.raw_get_value_top(array);
            let start = self.get_value_top(start).assume_int() as usize;
            let end = self.get_value_top(end).assume_int() as usize;
            let x = match &*list {
                RuntimeValue::List(l) => &l.items[start..end],
                RuntimeValue::Str(s) => {
                    let Some(x) = s.get(start..end)
                    else {
                        self.push_literal(RuntimeValue::Undefined);  // out of bounds
                        return;
                    };

                    self.push_literal(RuntimeValue::Str(x.to_string()));
                    return;
                }
                _ => std::hint::unreachable_unchecked(),
            };

            x
        };

        self.top_mut().indexes.extend_from_slice(x);
    }

    pub fn index_string_bytes(&mut self, string: usize, index: usize) {
        let str = self.get_value_top(string);
        match str {
            RuntimeValue::Str(s) => {
                let b = s.as_bytes()[index];
                let b = b as i64;
                let b = RuntimeValue::Int(b);
                self.push_literal(b);
            },
            _ => unsafe { std::hint::unreachable_unchecked() },
        }
    }

    pub fn append_array(&mut self, array: usize, index: usize) {
        self.extend_array(array, &[index]);
    }

    pub fn extend_array(&mut self, array: usize, indexes: &[usize]) {
        // FIXME: improve performance
        let indexes: Vec<usize> = indexes.iter().map(|f| self.top().indexes[*f]).collect();
        unsafe {
            let list = self.raw_get_value_top_mut(array);
            match &mut *list {
                RuntimeValue::List(l) => {
                    for abs in indexes {
                        self.arena.inc_refcount(abs);
                        l.items.push(abs);
                    }
                },
                _ => std::hint::unreachable_unchecked(),
            };
        }
    }
}
