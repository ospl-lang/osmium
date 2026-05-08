use crate::ast::{Expression, LValue};

#[derive(Debug, Clone)]
pub struct BinaryOp {
    pub left: Expression,
    pub right: Expression,
    pub kind: BinaryOpType
}

#[derive(Debug, Clone)]
pub struct AssignOp {
    pub left: LValue,
    pub right: Expression,
    pub kind: BinaryOpType
}

#[derive(Debug, Clone)]
pub enum BinaryOpType {
    /* mathematical */
    Add,
    Subtract,
    Multiply,
    Divide,
    Modulo,

    /* comparison */
    Equals,
    NotEquals,
    Gt,
    Lt,
    Ge,
    Le,
    Question,

    /* logical */
}

#[derive(Debug, Clone)]
pub enum UnaryOpType {
    LogicNot,
    Atsign,
    Decrement,
    Increment
}

#[derive(Debug, Clone)]
pub struct UnaryOp {
    pub expr: Expression,
    pub kind: UnaryOpType,
}