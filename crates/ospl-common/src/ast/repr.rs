// use crate::ast::{Entity, module::{VPath, VScope}};

use crate::ast::{frame::Scope, module::VScope};

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, PartialEq, Eq, Clone)]
pub enum Op {
    Add,
    Sub,
    Mul,
    Div,
    Mod,
}

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, PartialEq, Clone)]
pub enum Expr {
    StaticLiteral(StaticValue),
    BinaryOp {
        op: Op,
        lhs: Box<Expr>,
        rhs: Box<Expr>,
    },
    // Construct(VPath),
    Call(Box<Expr>, Vec<Expr>),
    LValue(LValue)
}

/// A value, in non-runtime representation.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, PartialEq, Clone)]
pub enum StaticValue {
    Int(i64),
    Float(f64),
    Str(String),
    Function(FunctionData),
    Frame(Scope),
    Nul
}

impl Into<Type> for StaticValue {
    fn into(self) -> Type {
        return match self {
            Self::Int(_) => Type::Int,
            Self::Float(_) => Type::Float,
            Self::Str(_) => Type::Str,
            Self::Function(fd) => Type::Function(fd.ty),
            Self::Frame(f) => Type::Frame(f),
            Self::Nul => Type::Nul,
        }
    }
}

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, PartialEq, Clone)]
pub enum Type {
    Int,
    Float,
    Str,
    Function(FunctionType),
    Module(VScope),
    Frame(Scope),
    Nul,
}

#[derive(Debug, PartialEq, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct FunctionData {
    pub code: Vec<Stmt>,
    pub ty: FunctionType
}

#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Parameter {
    pub ident: String,
    pub ty: Type,
}

impl PartialEq for Parameter {
    fn eq(&self, other: &Self) -> bool {
        return self.ty == other.ty
    }
}

#[derive(Debug, PartialEq, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct FunctionType {
    pub args: Vec<Parameter>,
    pub ret: Box<Type>,
}

#[derive(Debug, PartialEq, Eq, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum LValue {
    Var(String),
    Property(Box<LValue>, String),
}

#[derive(Debug, PartialEq, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum Stmt {
    Declare {
        lhs: LValue,
        rhs: Option<Expr>,
    },
    Assign {
        lhs: LValue,
        rhs: Expr,
    },
    Return(Expr),
    ReturnScope,
    Break,
    Continue,
}
