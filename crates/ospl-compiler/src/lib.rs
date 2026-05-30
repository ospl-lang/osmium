use std::sync::{Arc, Mutex, PoisonError};

use ospl_common::{ast::{Scope, Type, ops::BinaryOpType, spanning::UnknownLocation}, inst::symbols::DebugSymbolTable};

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
    pub symbols: Arc<Mutex<DebugSymbolTable>>,
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
    InternalError(Box<dyn std::error::Error>),
    Bug,
}

impl From<Box<dyn std::error::Error>> for CEData {
    fn from(value: Box<dyn std::error::Error>) -> Self {
        return Self::InternalError(value)
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

impl From<Box<dyn std::error::Error>> for CE {
    fn from(value: Box<dyn std::error::Error>) -> Self {
        return Self {
            at: Box::new(UnknownLocation),
            during: "unknown...",
            msg: None,
            error: CEData::from(value),
        }
    }
}

impl<T> From<PoisonError<std::sync::MutexGuard<'_, T>>> for CE {
    fn from(_: PoisonError<std::sync::MutexGuard<'_, T>>) -> Self {
        Self {
            at: Box::new(UnknownLocation),
            during: "mutex poisoned",
            msg: None,
            error: CEData::Bug,
        }
    }
}

pub mod ast;
mod impls;
// mod tests;