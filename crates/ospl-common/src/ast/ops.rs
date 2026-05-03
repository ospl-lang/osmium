use crate::ast::Expression;

#[derive(Debug)]
pub struct BinaryOp {
    pub left: Expression,
    pub right: Expression,
    pub kind: BinaryOpType
}

#[derive(Debug)]
pub enum BinaryOpType {
    Add,
    Subtract,
    Multiply,
    Divide,
    Modulo,
}
