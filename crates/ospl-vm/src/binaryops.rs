use std::hint::unreachable_unchecked;

use ospl_common::inst::{RT, list::List, make};

use crate::{VM, RuntimeValue};

// comparison
impl VM {
    fn get_two_regs(&self, a: usize, b: usize) -> (&RuntimeValue, &RuntimeValue) {
        let va = self.get_value_top(a);
        let vb = self.get_value_top(b);
        return (va, vb)
    }

    #[inline(always)]
    fn _eq_regs(&self, a: usize, b: usize) -> bool {
        let x = self.get_two_regs(a, b);
        return x.0 == x.1
    }

    #[inline(always)]
    fn _neq_regs(&self, a: usize, b: usize) -> bool {
        let x = self.get_two_regs(a, b);
        return x.0 != x.1
    }

    #[inline(always)]
    fn _gt_regs(&self, _a: usize, _b: usize) -> bool {
        // return match self.get_two_regs(a, b) {
        //     (RuntimeValue::Int(xa), RuntimeValue::Int(xb)) => xa > xb,
        //     (RuntimeValue::Address(xa), RuntimeValue::Address(xb)) => xa > xb,
        //     (RuntimeValue::Float(xa), RuntimeValue::Float(xb)) => xa > xb,
        //     (other1, other2) => panic!("cannot gt value {other1:?} with value {other2:?}!"),
        // }
        return false
    }

    pub fn eq_regs(&mut self, a: usize, b: usize) {
        let x = self._eq_regs(a, b);
        self.push_literal(make::bool(x));
    }

    pub fn neq_regs(&mut self, a: usize, b: usize) {
        let x = self._neq_regs(a, b);
        self.push_literal(make::bool(x));
    }

    pub fn gt_regs(&mut self, a: usize, b: usize) {
        let x = self._gt_regs(a, b);
        self.push_literal(make::bool(x));
    }

    pub fn lt_regs(&mut self, a: usize, b: usize) {
        let x = !self._gt_regs(a, b);
        self.push_literal(make::bool(x));
    }

    pub fn gte_regs(&mut self, a: usize, b: usize) {
        let x = self._gt_regs(a, b) | self._eq_regs(a, b);
        self.push_literal(make::bool(x));
    }

    pub fn lte_regs(&mut self, a: usize, b: usize) {
        let x = (!self._gt_regs(a, b)) | self._eq_regs(a, b);
        self.push_literal(make::bool(x));
    }

    pub fn neg_reg(&mut self, a: usize) {
        let t = self.get_truthiness(a);
        self.push_literal(make::bool(t));
    }
}

impl VM {
    #[inline(always)]
    fn _add_regs(&mut self, a: usize, b: usize) -> RuntimeValue {
        let (aa, bb) = self.get_two_regs(a, b);
        match (aa.tag, bb.tag) {
            (RT::Int, RT::Int) => make::int(unsafe { aa.data.int + bb.data.int }),
            (RT::Addr, RT::Addr) => make::addr(unsafe { aa.data.address + bb.data.address }),
            (RT::Float, RT::Float) => make::float(unsafe { aa.data.float + bb.data.float }),
            _ => unimplemented!()
        }
    }

    #[inline(always)]
    fn _sub_regs(&mut self, a: usize, b: usize) -> RuntimeValue {
        let (aa, bb) = self.get_two_regs(a, b);
        match (aa.tag, bb.tag) {
            (RT::Int, RT::Int) => make::int(unsafe { aa.data.int - bb.data.int }),
            (RT::Addr, RT::Addr) => make::addr(unsafe { aa.data.address - bb.data.address }),
            (RT::Float, RT::Float) => make::float(unsafe { aa.data.float - bb.data.float }),
            _ => unimplemented!()
        }
    }

    /// Find `b` in `a`
    pub fn question_mark(&mut self, a: usize, b: usize) {
        let (aa, bb) = self.get_two_regs(a, b);
        match (aa.tag, bb.tag) {
            (RT::List, _) => {
                let found = self.find_in_list(unsafe { &aa.data.list }, bb);
                if let Some(found) = found {
                    self.stack.top_add_index(found);
                } else {
                    self.push_literal(make::undefined(()));
                }
            },
            (u1, u2) => unimplemented!("unknown a?b op: {u1:?} and {u2:?}")
        }
    }

    pub fn find_in_list(&self, l: &List, v: &RuntimeValue) -> Option<usize> {
        let mut found = None;
        for item in &l.items {
            let i = self.get_value_top(*item);
            if i == v {
                // we found it
                found = Some(*item)
            }
        }

        return found
    }

    pub fn add_regs(&mut self, a: usize, b: usize) {
        let r = self._add_regs(a, b);
        self.push_literal(r);
    }

    pub fn sub_regs(&mut self, a: usize, b: usize) {
        let r = self._sub_regs(a, b);
        self.push_literal(r);
    }
}

// assign ops
impl VM {
    pub fn add_assign(&mut self, a: usize, b: usize) {
        debug_assert!(a != b);

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
                (RT::List, _) => {
                    self.append_array(a, b);
                },
                (RT::Str, RT::Str) => {
                    let a: &mut String = &mut aa.data.str;
                    let b: &String = &bb.data.str;

                    a.push_str(b);
                },
                // (RT::Str(s1), RuntimeValue::Char(c1)) => {
                //     s1.push(*c1);
                // },

                (err_a, err_b) => panic!("can't add-assign {:?} += {:?}", err_a, err_b),
                // _ => unreachable_unchecked()
            }
        }
    }

    pub fn sub_assign(&mut self, a: usize, b: usize) {
        debug_assert!(a != b);

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
