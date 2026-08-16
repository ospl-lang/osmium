use std::hint::unreachable_unchecked;

use ospl_common::inst::{RT, list::List, make};

use crate::{VM, RuntimeValue};

// comparison
impl VM {
    fn get_two_regs(&self, a: usize, b: usize) -> (&RuntimeValue, &RuntimeValue) {
        let va: &RuntimeValue = self.get_value_top(a);
        let vb = self.get_value_top(b);
        return (va, vb)
    }

    #[inline(always)]
    pub fn gt_regs(&mut self, a: usize, b: usize) {
        let (aa,bb) = self.get_two_regs(a, b);
        match (aa.tag, bb.tag) {
            (RT::List, _) => { self.push_literal(make::list({
                let mut x = unsafe { aa.data.list.items.clone() };
                x.push_back(b);

                List {
                    items: x
                }
            })); }
            _ => { self.push_literal(make::bool(aa > bb)); }
        }
    }

    #[inline(always)]
    pub fn lt_regs(&mut self, a: usize, b: usize) {
        let (aa,bb) = self.get_two_regs(a, b);
        match (aa.tag, bb.tag) {
            (RT::List, _) => { self.push_literal(make::list({
                let mut x = unsafe { aa.data.list.items.clone() };
                x.push_front(b);

                List {
                    items: x
                }
            })); }
            _ => { self.push_literal(make::bool(aa < bb)); }
        }
    }
    
    #[inline(always)]
    pub fn eq_regs(&mut self, a: usize, b: usize) {
        let (aa,bb) = self.get_two_regs(a, b);
        self.push_literal(make::bool(aa == bb));
    }

    #[inline(always)]
    pub fn neq_regs(&mut self, a: usize, b: usize) {
        let (aa,bb) = self.get_two_regs(a, b);
        self.push_literal(make::bool(aa != bb));
    }

    #[inline(always)]
    pub fn gte_regs(&mut self, a: usize, b: usize) {
        let (aa,bb) = self.get_two_regs(a, b);
        self.push_literal(make::bool(aa > bb || aa == bb));
    }

    #[inline(always)]
    pub fn lte_regs(&mut self, a: usize, b: usize) {
        let (aa,bb) = self.get_two_regs(a, b);
        self.push_literal(make::bool(aa < bb || aa == bb));
    }

    #[inline(always)]
    pub fn neg_reg(&mut self, a: usize) {
        let t = self.get_truthiness(a);
        self.push_literal(make::bool(t));
    }

    #[inline(always)]
    pub fn and_regs(&mut self, a: usize, b: usize) {
        let (aa,bb) = self.get_two_regs(a, b);
        let t = unsafe {
            match (aa.tag, bb.tag) {
                (RT::Bool, RT::Bool) => make::bool(aa.data.bool && bb.data.bool),
                (RT::Int, RT::Int) => make::int(aa.data.int & bb.data.int),
                (RT::Addr, RT::Addr) => make::addr(aa.data.address & bb.data.address),
                _ => unimplemented!()
            }
        };
        self.push_literal(t);
    }

    #[inline(always)]
    pub fn or_regs(&mut self, a: usize, b: usize) {
        let (aa,bb) = self.get_two_regs(a, b);
        let t = unsafe {
            match (aa.tag, bb.tag) {
                (RT::Bool, RT::Bool) => make::bool(aa.data.bool || bb.data.bool),
                (RT::Int, RT::Int) => make::int(aa.data.int | bb.data.int),
                (RT::Addr, RT::Addr) => make::addr(aa.data.address | bb.data.address),
                _ => unimplemented!()
            }
        };
        self.push_literal(t);
    }

    #[inline(always)]
    pub fn xor_regs(&mut self, a: usize, b: usize) {
        let (aa,bb) = self.get_two_regs(a, b);
        let t = unsafe {
            match (aa.tag, bb.tag) {
                (RT::Bool, RT::Bool) => make::bool(aa.data.bool ^ bb.data.bool),
                (RT::Int, RT::Int) => make::int(aa.data.int ^ bb.data.int),
                (RT::Addr, RT::Addr) => make::addr(aa.data.address ^ bb.data.address),
                _ => unimplemented!()
            }
        };
        self.push_literal(t);
    }
}

impl VM {
    #[inline(always)]
    pub fn add_regs(&mut self, a: usize, b: usize) {
        let (aa, bb) = self.get_two_regs(a, b);
        let x = match (aa.tag, bb.tag) {
            (RT::Int, RT::Int) => make::int(unsafe { aa.data.int + bb.data.int }),
            (RT::Addr, RT::Addr) => make::addr(unsafe { aa.data.address + bb.data.address }),
            (RT::Float, RT::Float) => make::float(unsafe { aa.data.float + bb.data.float }),
            (RT::Str, RT::Str) => make::str(unsafe {
                let mut s = aa.data.str.clone();
                s.push_str(&bb.data.str);
                s.to_string()
            }),
            other => unimplemented!("{other:?}")
        };

        self.push_literal(x);
    }

    #[inline(always)]
    pub fn sub_regs(&mut self, a: usize, b: usize) {
        unsafe {
            let va_ptr = self.raw_get_value_top_mut(a);
            let vb_ptr = self.raw_get_value_top(b);
            let aa = &mut *va_ptr;
            let bb = &*vb_ptr;

            // it's fine don't worry about it
            //   -- Amber

            match (aa.tag, bb.tag) {
                (RT::Int, RT::Int) => {self.push_literal(make::int(aa.data.int - bb.data.int)); },
                (RT::Addr, RT::Addr) => {self.push_literal(make::addr(aa.data.address - bb.data.address)); },
                (RT::Float, RT::Float) => {self.push_literal(make::float(aa.data.float - bb.data.float)); },
                (RT::List, RT::Addr) => {
                    let x = bb.data.address;
                    if (*aa.data.list).items.len() == 0 {
                        self.push_literal(make::undefined(()));
                    }
                    self.stack.top_add_index((*aa.data.list).items.remove(x as usize).unwrap())
                },
                (RT::List, RT::Int) => {
                    let x = bb.data.int;
                    if (*aa.data.list).items.len() == 0 {
                        self.push_literal(make::undefined(()));
                    }
                    self.stack.top_add_index((*aa.data.list).items.remove(x as usize).unwrap())
                },
                _ => unimplemented!()
            };
        }
    }

    #[inline(always)]
    pub fn div_regs(&mut self, a: usize, b: usize) {
        let (aa, bb) = self.get_two_regs(a, b);
        let x = match (aa.tag, bb.tag) {
            (RT::Int, RT::Int) => make::int(unsafe { aa.data.int / bb.data.int }),
            (RT::Addr, RT::Addr) => make::addr(unsafe { aa.data.address / bb.data.address }),
            (RT::Float, RT::Float) => make::float(unsafe { aa.data.float / bb.data.float }),
            _ => unimplemented!()
        };

        self.push_literal(x);
    }

    #[inline(always)]
    pub fn mul_regs(&mut self, a: usize, b: usize) {
        let (aa, bb) = self.get_two_regs(a, b);
        let x = match (aa.tag, bb.tag) {
            (RT::Int, RT::Int) => make::int(unsafe { aa.data.int * bb.data.int }),
            (RT::Addr, RT::Addr) => make::addr(unsafe { aa.data.address * bb.data.address }),
            (RT::Float, RT::Float) => make::float(unsafe { aa.data.float * bb.data.float }),
            _ => unimplemented!()
        };

        self.push_literal(x);
    }

    #[inline(always)]
    pub fn mod_regs(&mut self, a: usize, b: usize) {
        let (aa, bb) = self.get_two_regs(a, b);
        let x = match (aa.tag, bb.tag) {
            (RT::Int, RT::Int) => make::int(unsafe { aa.data.int % bb.data.int }),
            (RT::Addr, RT::Addr) => make::addr(unsafe { aa.data.address % bb.data.address }),
            (RT::Float, RT::Float) => make::float(unsafe { aa.data.float % bb.data.float }),
            _ => unimplemented!()
        };

        self.push_literal(x);
    }

    #[inline(always)]
    pub fn lshift_regs(&mut self, a: usize, b: usize) {
        // SAFETY: I WILL NOT MUTATE aa AND bb FROM ANOTHER THREAD
        // (I think that's the only safety, and we'll never see that)

        let (aa, bb) = self.get_two_regs(a, b);
        let x = match (aa.tag, bb.tag) {
            (RT::Int, RT::Int) => make::int(unsafe { aa.data.int << bb.data.int }),
            (RT::Addr, RT::Addr) => make::addr(unsafe { aa.data.address << bb.data.address }),
            (RT::List, _) => make::addr(self.append_array_front(a, b) as u64),
            _ => unimplemented!()
        };

        self.push_literal(x);
    }

    #[inline(always)]
    pub fn rshift_regs(&mut self, a: usize, b: usize) {
        // SAFETY: I WILL NOT MUTATE aa AND bb FROM ANOTHER THREAD
        // (I think that's the only safety, and we'll never see that)

        let (aa, bb) = self.get_two_regs(a, b);
        let x = match (aa.tag, bb.tag) {
            (RT::Int, RT::Int) => make::int(unsafe { aa.data.int >> bb.data.int }),
            (RT::Addr, RT::Addr) => make::addr(unsafe { aa.data.address >> bb.data.address }),
            (RT::List, _) => make::addr(self.append_array_back(a, b) as u64),
            _ => unimplemented!()
        };

        self.push_literal(x);
    }
}

// assign ops
impl VM {
    pub fn add_assign(&mut self, a: usize, b: usize) {
        // SAFETY: I guarantee that `a` and `b` are not the same value
        unsafe {
            let va_ptr = self.raw_get_value_top_mut(a);
            let vb_ptr = self.raw_get_value_top(b);

            // Dereference the pointers first
            let aa = &mut *va_ptr;
            let bb = &*vb_ptr;

            // match aa.tag {
            match (aa.tag, bb.tag) {
                (RT::Int, RT::Int) => {
                    aa.data.int += bb.data.int;
                },
                (RT::Addr, RT::Addr) => {
                    aa.data.address += bb.data.address;
                },
                (RT::Str, RT::Str) => {
                    let a: &mut String = &mut aa.data.str;
                    let b: &String = &bb.data.str;

                    a.push_str(b);
                },
                (RT::Str, RT::Char) => {
                    let a: &mut String = &mut aa.data.str;
                    let b: &char = &bb.data.char;

                    a.push(*b);
                }
                // (RT::Str(s1), RuntimeValue::Char(c1)) => {
                //     s1.push(*c1);
                // },

                (err_a, err_b) => panic!("can't add-assign {:?} += {:?}", err_a, err_b),
                // _ => unreachable_unchecked()
            }
        }
    }

    pub fn sub_assign(&mut self, a: usize, b: usize) {
        // SAFETY: I guarantee that `a` and `b` are not the same value
        unsafe {
            let va_ptr = self.raw_get_value_top_mut(a);
            let vb_ptr = self.raw_get_value_top(b);

            // Dereference the pointers first
            let va = &mut *va_ptr;
            let vb = &*vb_ptr;

            match (va.tag, vb.tag) {
                (RT::Int, RT::Int) => va.data.int -= vb.data.int,
                (RT::List, RT::Addr) => {
                    let u = vb.data.address;
                    let l = &mut va.data.list;
                    l.items.remove(u as usize);
                },
                (RT::Str, RT::Addr) => {
                    let a: &mut String = &mut va.data.str;
                    let b = vb.data.address;
                    a.remove(b as usize);
                },

                // (err_a, err_b) => panic!("can't op-assign {:?} += {:?}", err_a, err_b),
                (_, _) => unreachable_unchecked()
            }
        }
    }
}
