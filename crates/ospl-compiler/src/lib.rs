use ospl_common::ast::{Scope, Type};

pub type RelativeVarID = usize;

/// All nested scopes during compilation.
#[derive(Default)]
struct ScopeStack {
    scopes: Vec<Scope>,
}

pub struct Compiler {
    stack: ScopeStack
}

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