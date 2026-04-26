use crate::{VM, Value};

macro_rules! op {
    (
        binary,
        $a:ident,
        $b:ident,
        $op:tt,
        { $($extra_arms:tt)* }
    ) => {
        match ($a, $b) {
            (Value::Int(xa), Value::Int(xb)) => Value::Int(xa $op xb),
            (Value::Float(xa), Value::Float(xb)) => Value::Float(xa $op xb),
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
                    +,
                    { $($extra_arms)* }
                );

                self.push_literal(r);
            }
        }
    };
}

impl VM {
    pub fn eq_regs(&mut self, a: usize, b: usize) {
        let va = self.get_value_top(a);
        let vb = self.get_value_top(b);

        let r = match (va, vb) {
            (Value::Int(xa), Value::Int(xb)) => Value::Bool(xa == xb),
            (Value::Float(xa), Value::Float(xb)) => Value::Bool(xa == xb),
            _ => panic!("cannot eq these two!"),
        };

        self.push_literal(r);
    }
}

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
                (Value::Int(x), Value::Int(y)) => *x += *y,
                (Value::List(x), all) => {
                    let idx = self.arena.push(all.clone_composite());
                    x.push(idx, &mut self.arena);
                }

                (err_a, err_b) => panic!("can't add-assign {:?} += {:?}", err_a, err_b),
            }
        }
    }
}
