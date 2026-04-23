use crate::ast::{class::{CTable, instance::{InstanceType, Entity}}, module::{VPath, VScope}};

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
    Construct(VPath),
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
    Class(CTable),
    Nul
}

impl Into<Type> for StaticValue {
    fn into(self) -> Type {
        return match self {
            Self::Int(_) => Type::Int,
            Self::Float(_) => Type::Float,
            Self::Str(_) => Type::Str,
            Self::Function(fd) => Type::Function(fd.ty),
            Self::Class(ct) => Type::Class(ct),
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
    Class(CTable),
    Instance(InstanceType),
    Module(VScope),
    Nul,
}

impl Type {
    pub fn get_property(&self, name: &str) -> Option<&dyn Entity> {
        return match self {
            Type::Instance(i) => i.values.get(name).map(|x| x as &dyn Entity),
            _ => None,
        }
    }

    // pub fn mut_property(&mut self, name: &str) -> Option<&mut dyn Entity> {
    //     return match self {
    //         Type::Instance(i) => i.values.get_mut(name).map(|x| x as &mut dyn Entity),
    //         _ => None,
    //     }
    // }
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
    pub ret: Box<Type>
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
    DeclareReal {
        lhs: LValue,
        rhs: Option<Expr>,
    },
    AssignReal {
        lhs: LValue,
        rhs: Expr,
    },
    DeclareType {
        vp: VPath,
        rhs: Type,
    },
    Return(Expr),
    Break,
    Continue,
}
