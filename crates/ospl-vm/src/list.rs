use ospl_common::inst::{RT, assume, assume_mut, list::List, make};

use crate::VM;

impl VM {
    pub fn push_array(&mut self, indexes: &[usize]) {
        let mut list = List::default();
        for rel in indexes {
            let abs = self.stack.top_indexes()[*rel];
            
            list.items.push(abs);
        }

        self.push_literal(make::list(list));
    }

    pub fn index(&mut self, array: usize, index: usize) {
        // SAFETY: the invariant here is that the instruction is operating on a
        // valid type and the index into the array is valid.
        let x = {
            let indexable = self.get_value_top(array);
            let index = assume::address(self.get_value_top(index)).unwrap();
            let x = match indexable.tag {
                RT::List => unsafe {
                    if let Some(d) = indexable.data.list.items.get(*index as usize) {
                        *d
                    } else {
                        self.push_literal(make::undefined(()));
                        return  // the function
                    }
                },
                RT::Str => {
                    let Some(x) = (unsafe { indexable.data.str.chars().nth(*index as usize) })
                    else {
                        self.push_literal(make::undefined(()));  // out of bounds
                        return;
                    };

                    self.push_literal(make::char(x));
                    return;
                },
                _ => unsafe { std::hint::unreachable_unchecked() },
            };

            x
        };
        self.stack.top_add_index(x);
    }

    pub fn slice(&mut self, _array: usize, _start: usize, _end: usize) {
        // let _x = unsafe {
        //     let val = self.raw_get_value_top(array);
        //     let start = self.get_value_top(start).assume_int() as usize;
        //     let end = self.get_value_top(end).assume_int() as usize;
        //     let x = match (&*val).tag {
        //         RT::List => unsafe { &(*val).data.list.items[start..end] },
        //         RuntimeValue::Str(s) => {
        //             let Some(x) = s.get(start..end)
        //             else {
        //                 self.push_literal(RuntimeValue::Undefined);  // out of bounds
        //                 return;
        //             };

        //             self.push_literal(RuntimeValue::Str(x.to_string()));
        //             return;
        //         }
        //         _ => std::hint::unreachable_unchecked(),
        //     };

        //     x
        // };

        unimplemented!("slicing isn't fully imeplemented")
    }

    pub fn index_string_bytes(&mut self, string: usize, index: usize) {
        let str = self.get_value_top(string);
        match assume::str(str) {
            Some(s) => {
                let b = s.as_bytes()[index];
                let b = b as i64;
                let b = make::int(b);
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
        let indexes: Vec<usize> = indexes.iter().map(|f| self.stack.top_indexes()[*f]).collect();
        unsafe {
            let list = self.raw_get_value_top_mut(array);
            match assume_mut::list(&mut *list) {
                Some(l) => {
                    for abs in indexes {
                        l.items.push(abs);
                    }
                },
                _ => std::hint::unreachable_unchecked(),
            };
        }
    }
}
