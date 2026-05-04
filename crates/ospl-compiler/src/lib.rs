use ospl_common::ast::{Scope, Type};

pub type RelativeVarID = usize;

/// All nested scopes during compilation.
#[derive(Debug, Default)]
struct ScopeStack {
    scopes: Vec<Scope>,
}

#[derive(Debug)]
pub struct Compiler {
    stack: ScopeStack
}

#[derive(Debug)]
pub struct EvalResult {
    address: RelativeVarID,
    ty: Type,
}

#[derive(Debug)]
pub enum CompErr {
    MismatchedTypes {
        expected: Type,
        got: Type
    },
    WrongArgCount {
        expected: usize,
        got: usize,
    },
    NotFoundInScope {
        needed: String,
    },
    NoScopeToCapture,
}

pub type Res<T> = Result<T, CompErr>;

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
mod tests;