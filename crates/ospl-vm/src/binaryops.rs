use ospl_common::inst::list::List;

use crate::{VM, RuntimeValue};

// comparison
impl VM {
    fn get_two_regs(&self, a: usize, b: usize) -> (&RuntimeValue, &RuntimeValue) {
        let va = self.get_value_top(a);
        let vb = self.get_value_top(b);
        return (va, vb)
    }

    fn _eq_regs(&self, a: usize, b: usize) -> bool {
        return match self.get_two_regs(a, b) {
            (RuntimeValue::Int(xa), RuntimeValue::Int(xb)) => xa == xb,
            (RuntimeValue::Address(xa), RuntimeValue::Address(xb)) => xa == xb,
            (RuntimeValue::Float(xa), RuntimeValue::Float(xb)) => xa == xb,
            (RuntimeValue::Undefined, RuntimeValue::Undefined) => true,
            (_, RuntimeValue::Undefined) => false,
            (other1, other2) => panic!("cannot eq value {other1:?} with value {other2:?}!"),
        }
    }

    fn _gt_regs(&self, a: usize, b: usize) -> bool {
        return match self.get_two_regs(a, b) {
            (RuntimeValue::Int(xa), RuntimeValue::Int(xb)) => xa > xb,
            (RuntimeValue::Address(xa), RuntimeValue::Address(xb)) => xa > xb,
            (RuntimeValue::Float(xa), RuntimeValue::Float(xb)) => xa > xb,
            (other1, other2) => panic!("cannot gt value {other1:?} with value {other2:?}!"),
        }
    }

    pub fn eq_regs(&mut self, a: usize, b: usize) {
        let x = self._eq_regs(a, b);
        self.push_literal(RuntimeValue::Bool(x));
    }

    pub fn neq_regs(&mut self, a: usize, b: usize) {
        let x = !self._eq_regs(a, b);
        self.push_literal(RuntimeValue::Bool(x));
    }

    pub fn gt_regs(&mut self, a: usize, b: usize) {
        let x = self._gt_regs(a, b);
        self.push_literal(RuntimeValue::Bool(x));
    }

    pub fn lt_regs(&mut self, a: usize, b: usize) {
        let x = !self._gt_regs(a, b);
        self.push_literal(RuntimeValue::Bool(x));
    }
}

impl VM {
    fn _add_regs(&mut self, a: usize, b: usize) -> RuntimeValue {
        match self.get_two_regs(a, b) {
            (RuntimeValue::Int(xa), RuntimeValue::Int(xb)) => RuntimeValue::Int(*xa + *xb),
            (RuntimeValue::Address(xa), RuntimeValue::Address(xb)) => RuntimeValue::Address(*xa + *xb),
            (RuntimeValue::Float(xa), RuntimeValue::Float(xb)) => RuntimeValue::Float(*xa + *xb),
            _ => unimplemented!()
        }
    }

    fn _sub_regs(&mut self, a: usize, b: usize) -> RuntimeValue {
        match self.get_two_regs(a, b) {
            (RuntimeValue::Int(xa), RuntimeValue::Int(xb)) => RuntimeValue::Int(*xa - *xb),
            (RuntimeValue::Address(xa), RuntimeValue::Address(xb)) => RuntimeValue::Address(*xa - *xb),
            (RuntimeValue::Float(xa), RuntimeValue::Float(xb)) => RuntimeValue::Float(*xa - *xb),
            _ => unimplemented!()
        }
    }

    /// Find `b` in `a`
    pub fn question_mark(&mut self, a: usize, b: usize) {
        match self.get_two_regs(a, b) {
            (RuntimeValue::List(l), v) => {
                let found = self.find_in_list(l, v);
                if let Some(found) = found {
                    self.top_mut().indexes.push(found);
                } else {
                    self.push_literal(RuntimeValue::Undefined);
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
            let va = &mut *va_ptr;
            let vb = &*vb_ptr;

            match (va, vb) {
                (RuntimeValue::Int(x), RuntimeValue::Int(y)) => *x += *y,
                (RuntimeValue::List(_), RuntimeValue::List(r)) => {
                    self.extend_array(a, &r.items);
                },
                (RuntimeValue::List(_), _) => {
                    self.append_array(a, b);
                },
                (RuntimeValue::Str(s1), RuntimeValue::Str(s2)) => {
                    s1.push_str(&*s2);
                },

                (err_a, err_b) => panic!("can't add-assign {:?} += {:?}", err_a, err_b),
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

            match (va, vb) {
                (RuntimeValue::Int(x), RuntimeValue::Int(y)) => *x -= *y,
                (RuntimeValue::List(l), v) => {
                    let found = self.find_in_list(l, v);
                    if let Some(found) = found {
                        l.items.remove(found);
                    }
                },
                (RuntimeValue::Str(s1), RuntimeValue::Str(s2)) => {
                    s1.push_str(&*s2);
                },

                (err_a, err_b) => panic!("can't op-assign {:?} += {:?}", err_a, err_b),
            }
        }
    }
}
