//! Functions to do with types

use ospl_common::{ast::Type, inst::RuntimeValue};

use crate::VM;
impl VM {
    pub fn cast_value(&mut self, v: usize, ty: usize) {
        let ty = Type::from_primitive_type_id(ty);
        let v = self.get_value_top(v);
        let new = match (v, ty) {
            (RuntimeValue::Int(i), Type::Address) => RuntimeValue::Address(*i as u64),
            (RuntimeValue::Address(i), Type::Int) => RuntimeValue::Int(*i as i64),

            (RuntimeValue::Float(f), Type::Int) => RuntimeValue::Int(*f as i64),
            (RuntimeValue::Int(i), Type::Float) => RuntimeValue::Float(*i as f64),

            (RuntimeValue::Address(u), Type::Float) => RuntimeValue::Float(*u as f64),
            (RuntimeValue::Float(f), Type::Address) => RuntimeValue::Address(*f as u64),

            (RuntimeValue::Address(u), Type::Char) => RuntimeValue::Char(*u as u8 as char),
            (RuntimeValue::Char(c), Type::Address) => RuntimeValue::Address(*c as u64),

            (r, t) => panic!("unknown type conversion {r:?} into type {t:?}")
        };

        self.push_literal(new);
    }
}