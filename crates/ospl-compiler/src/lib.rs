use std::fmt::Debug;

use ospl_common::ast::{Scope, Type, ops::BinaryOpType};

pub type RelativeVarID = usize;

pub mod package;

/// All nested scopes during compilation.
#[derive(Debug, Default)]
pub struct ScopeStack {
    scopes: Vec<Scope<Type>>,
}

#[derive(Debug)]
pub struct Compiler {
    pub stack: ScopeStack,
}

#[derive(Debug, Clone)]
pub struct EvalResult {
    address: RelativeVarID,
    ty: Type,
}

#[derive(Debug)]
pub enum TypeExpectation {
    Exact(Type),
    AnyList,
    Indexable,
    Slicable,
    AnyScope,
}

#[derive(Debug)]
pub enum CEData {
    MismatchedTypes {
        expected: TypeExpectation,
        got: Type
    },
    UnresolvableUType {
        // ew but have to do it...
        t: Box<dyn std::any::Any>
    },
    WrongArgCount {
        expected: usize,
        got: usize,
    },
    NotFoundInScope {
        needed: String,
        scope: Scope<Type>,
    },
    NoScopeToCapture,
    InvalidOpForType {
        op: BinaryOpType,
        ty: Type
    },
    InvalidAssignOp {
        op: BinaryOpType,
    },
    UnrecognizedSpecialVar {
        ty: Type,
        special: String,
    },
    UncallableType {
        ty: Type,
    }
}

#[derive(Debug)]
pub struct CE {
    pub at: Box<dyn ospl_common::ast::spanning::Spannable>,
    pub error: CEData,
    pub during: &'static str,
    pub msg: Option<&'static str>,
}

pub type Res<T> = Result<T, CE>;

impl From<(usize, &Type)> for EvalResult {
    fn from(value: (usize, &Type)) -> Self {
        return Self {
            address: value.0,
            ty: value.1.clone()
        }
    }
}

pub mod ast;
mod impls;
// mod tests;