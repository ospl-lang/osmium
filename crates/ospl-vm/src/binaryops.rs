use crate::{VM, RuntimeValue};

macro_rules! op {
    (
        binary,
        $a:ident,
        $b:ident,
        $op:tt,
        { $($extra_arms:tt)* }
    ) => {
        match ($a, $b) {
            (RuntimeValue::Int(xa), RuntimeValue::Int(xb)) => RuntimeValue::Int(xa $op xb),
            (RuntimeValue::Float(xa), RuntimeValue::Float(xb)) => RuntimeValue::Float(xa $op xb),
            $($extra_arms)*
            (invalid_a, invalid_b) => panic!(
                "invalid op (binary): {:?} {} {:?}",
                invalid_a,
                stringify!($op),
                invalid_b
            ),
        }
    };
}

macro_rules! impl_op {
    (
        $type:ident,
        $on:ident,
        $fname:ident,
        $op:tt,
        { $($extra_arms:tt)* }
    ) => {
        impl $on {
            #[inline(always)]
            pub fn $fname(&mut self, a: usize, b: usize) {
                let va = self.arena.get(self.top().indexes[a]);
                let vb = self.arena.get(self.top().indexes[b]);

                let r = op!(
                    $type,
                    va,
                    vb,
                    $op,
                    { $($extra_arms)* }
                );

                self.push_literal(r);
            }
        }
    };
}

impl VM {
    pub fn eq_regs_b(&mut self, a: usize, b: usize) -> bool {
        let va = self.get_value_top(a);
        let vb = self.get_value_top(b);

        let r = match (va, vb) {
            (RuntimeValue::Int(xa), RuntimeValue::Int(xb)) => xa == xb,
            (RuntimeValue::Float(xa), RuntimeValue::Float(xb)) => xa == xb,
            (other1, other2) => panic!("cannot eq value {other1:?} with value {other2:?}!"),
        };

        return r
    }

    pub fn eq_regs(&mut self, a: usize, b: usize) {
        let x = self.eq_regs_b(a, b);
        self.push_literal(RuntimeValue::Bool(x));
    }

    pub fn neq_regs(&mut self, a: usize, b: usize) {
        let x = !self.eq_regs_b(a, b);
        self.push_literal(RuntimeValue::Bool(x));
    }

    // can Gt / Le regs by doing subtractions
}

// math
impl_op!(binary, VM, add_regs, +, {});
impl_op!(binary, VM, sub_regs, -, {});
impl_op!(binary, VM, mul_regs, *, {});
impl_op!(binary, VM, div_regs, /, {});
impl_op!(binary, VM, mod_regs, %, {});

// TODO: implement assign-ops in impl_op!()

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
}
