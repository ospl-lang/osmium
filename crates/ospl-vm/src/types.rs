//! Functions to do with types

use ospl_common::{ast::Type, inst::{RT, make}};

use crate::VM;
impl VM {
    pub fn cast_value(&mut self, vi: usize, ty: usize) {
        let ty = Type::from_primitive_type_id(ty);
        let vv = self.get_value_top(vi);
        let new = unsafe { match (vv.tag, ty) {
            // numbers to numbers
            (RT::Int, Type::Address) => make::addr(vv.data.int as u64),
            (RT::Addr, Type::Int) => make::int(vv.data.address as i64),

            (RT::Float, Type::Int) => make::int(vv.data.float as i64),
            (RT::Int, Type::Float) => make::float(vv.data.int as f64),

            (RT::Addr, Type::Float) => make::float(vv.data.address as f64),
            (RT::Float, Type::Address) => make::addr(vv.data.float as u64),

            // numbers to strings
            (RT::Int, Type::Str) => make::str(vv.data.int.to_string()),
            (RT::Float, Type::Str) => make::str(vv.data.float.to_string()),
            (RT::Addr, Type::Str) => make::str(vv.data.address.to_string()),

            // special
            (RT::Addr, Type::Char) => make::char(vv.data.address as u8 as char),
            (RT::Char, Type::Address) => make::addr(vv.data.char as u64),

            (_, Type::Bool) => make::bool(self.get_truthiness(vi)),

            (r, t) => panic!("unknown type conversion {r:?} into type {t:?}")
        } };

        self.push_literal(new);
    }
}