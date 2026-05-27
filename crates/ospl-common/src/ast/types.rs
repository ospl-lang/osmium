use std::fmt::Debug;

use crate::ast::Scope;

impl Type {
    pub fn to_primitive_type_id(&self) -> usize {
        match self {
            Self::Nul => 0,
            Self::Int => 1,
            Self::Address => 2,
            Self::Float => 3,
            Self::Str => 4,
            Self::Undefined => 5,
            Self::Char => 6,

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
            6 => Self::Char,

            t => panic!("unknown primitive type ID: {t}")
        }
    }

    pub fn is_indexable(&self) -> bool {
        return matches!(self, Self::Str | Self::List(_))
    }

    pub fn is_sliceable(&self) -> bool {
        return matches!(self, Self::Str | Self::List(_))
    }
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FunctionType<T> {
    /// Relative var ID
    // pub captures: Vec<usize>,

    pub generics: Vec<T>,
    pub args: Vec<T>,
    pub ret: T,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum UType {
    PreResolved(Type),

    Function(Box<FunctionType<Self>>),
    List(Box<Self>),
    Scope(Scope<Self>),
    AnyScope,
    Typeof(String),
    ReturnTypeof(Box<Self>)
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Type {
    Nul, Undefined,

    /// A type-erased type.
    /// 
    /// It has an unknown runtime type. It cannot be operated upon but any
    /// creator of the value is aware as to what type it is and can fully
    /// utilize it.
    Unknown,

    Int, Address, Float, Char, Str, Bool, List(Box<Self>),
    Scope(Scope<Self>),
    ResolvingFunction(Box<FunctionType<AType>>),
    CallableFunction(Box<FunctionType<Self>>),

    ForeignLibrary,
    ForeignFunction(Vec<Self>, Box<Self>),
}

impl From<Type> for UType {
    fn from(value: Type) -> Self {
        return Self::PreResolved(value)
    }
}

/// The type used for function literals in the compiler internally
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AType {
    Resolved(Type),
    Generic(usize),
    Scope(Scope<Self>),
}