use crate::ast::Expression;

#[derive(Debug)]
pub struct BinaryOp {
    pub left: Expression,
    pub right: Expression,
    pub kind: BinaryOpType
}

#[derive(Debug)]
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

    /* logical */
}
