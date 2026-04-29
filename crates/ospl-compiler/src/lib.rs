use std::collections::HashMap;

use crate::ast::FunctionType;

pub type RelativeVarID = usize;

#[derive(Debug, Clone, PartialEq)]
pub enum Type {
    Nul, Undefined,
    Int,
    Scope(Scope),
    Function(Box<FunctionType>)
}

/// A single block of variables.
#[derive(Debug, Clone, Default)]
pub struct Scope {
    map: HashMap<String, Store>,

    // very private
    next_id: RelativeVarID,
}

#[derive(Clone, Debug)]
pub struct Store {
    ty: Type,
    var: RelativeVarID
}

impl PartialEq for Store {
    fn eq(&self, other: &Self) -> bool {
        return self.ty == other.ty
    }
}

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

pub mod ast;
mod impls;
mod tests;