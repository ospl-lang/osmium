//! Functions to do with types

use ospl_common::{ast::Type, inst::{RT, make}};

use crate::VM;
impl VM {
    pub fn cast_value(&mut self, v: usize, ty: usize) {
        let ty = Type::from_primitive_type_id(ty);
        let v = self.get_value_top(v);
        let new = unsafe { match (v.tag, ty) {
            // numbers to numbers
            (RT::Int, Type::Address) => make::addr(v.data.int as u64),
            (RT::Addr, Type::Int) => make::int(v.data.address as i64),

            (RT::Float, Type::Int) => make::int(v.data.float as i64),
            (RT::Int, Type::Float) => make::float(v.data.int as f64),

            (RT::Addr, Type::Float) => make::float(v.data.address as f64),
            (RT::Float, Type::Address) => make::addr(v.data.float as u64),

            // numbers to strings
            (RT::Int, Type::Str) => make::str(v.data.int.to_string()),
            (RT::Float, Type::Str) => make::str(v.data.float.to_string()),
            (RT::Addr, Type::Str) => make::str(v.data.address.to_string()),

            // special
            (RT::Addr, Type::Char) => make::char(v.data.address as u8 as char),
            (RT::Char, Type::Address) => make::addr(v.data.char as u64),

            (r, t) => panic!("unknown type conversion {r:?} into type {t:?}")
        } };

        self.push_literal(new);
    }
}