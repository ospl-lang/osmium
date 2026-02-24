use crate::base::{Scope, Type};

pub mod base;
pub mod optimization;
mod impls;

pub mod tests;

pub struct Compiler {
    scopes: Vec<Scope>,
}

impl Default for Compiler {
    fn default() -> Self {
        return Self {
            scopes: vec![Scope::default()],
        }
    }
}

impl Compiler {
    pub fn top(&self) -> &Scope {
        return self.scopes.last()
            .expect("invalid data passed in: at least one scope is required")
    }

    pub fn top_mut(&mut self) -> &mut Scope {
        return self.scopes.last_mut()
            .expect("invalid data passed in: at least one scope is required")
    }

    pub fn new_scope(&mut self) {
        self.scopes.push(Scope::default());
    }

    pub fn push_scope(&mut self, s: Scope) {
        self.scopes.push(s);
    }
}

pub struct BinaryOpEval {
    pub left: usize,
    pub right: usize,
    pub result: usize,
    pub ty: Type,
}

impl Into<Eval> for BinaryOpEval {
    fn into(self) -> Eval {
        return Eval {
            index: self.result,
            ty: self.ty
        }
    }
}

impl From<base::Local> for Eval {
    fn from(value: base::Local) -> Self {
        return Self {
            index: value.register,
            ty: value.ty
        }
    }
}

pub struct Eval {
    pub ty: Type,
    pub index: usize,
}

impl Eval {
    pub fn with_index(index: usize) -> Self {
        return Self {
            ty: Type::default(),
            index
        }
    }

    pub fn new(index: usize, ty: Type) -> Self {
        return Self { index, ty }
    }
}