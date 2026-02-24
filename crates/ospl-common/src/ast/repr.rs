use std::collections::HashMap;

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum Op {
    Add,
    Sub,
    Mul,
    Div,
    Mod,
}

#[derive(Debug, PartialEq, Clone)]
pub enum Expr {
    StaticLiteral(StaticValue),
    BinaryOp {
        op: Op,
        lhs: Box<Expr>,
        rhs: Box<Expr>,
    },
    Call(Box<Expr>, Vec<Expr>),
    LValue(LValue)
}

/// A value, in non-runtime representation.
#[derive(Debug, PartialEq, Clone)]
pub enum StaticValue {
    Int(i64),
    Float(f64),
    Function(FunctionData),
    Structure(StructureData),
    Nul
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum StaticType {
    Int,
    Float,
    Function,
    Structure(StructureData),
    Nul,
}

#[derive(Debug, PartialEq, Clone)]
pub struct FunctionData {
    pub code: Vec<Stmt>,
    pub args: Vec<StaticType>,
    pub ret: StaticType
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct StructureData {
    pub items: HashMap<String, StructItem>
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct StructItem {
    pub ty: StaticType,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum LValue {
    Var(String),
}

#[derive(Debug, PartialEq, Clone)]
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
    Break,
    Continue,
}
