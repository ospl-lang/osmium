use ospl_common::ast::{Scope, Type, ops::BinaryOpType};

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
pub enum TypeExpectation {
    Exact(Type),
    AnyList,
    AnyScope,
}

#[derive(Debug)]
pub enum CEData {
    MismatchedTypes {
        expected: TypeExpectation,
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
    }
}

#[derive(Debug)]
pub struct CE {
    pub at: Box<dyn ospl_common::ast::spanning::Spannable>,
    pub error: CEData,
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
mod tests;