use crate::ast::Type;

impl Type {
    pub fn to_primitive_type_id(&self) -> usize {
        match self {
            Self::Nul => 0,
            Self::Int => 1,
            Self::Address => 2,
            Self::Float => 3,
            Self::Str => 4,
            Self::Undefined => 5,

            t => panic!("cannot call .to_primitive_type_id() on a non-primitive type {t:?}"),
        }
    }

    pub fn from_primitive_type_id(p: usize) -> Self {
        match p {
            0 => Self::Nul,
            1 => Self::Int,
            2 => Self::Address,
            3 => Self::Float,
            4 => Self::Str,
            5 => Self::Undefined,
            t => panic!("unknown primitive type ID: {t}")
        }
    }
}