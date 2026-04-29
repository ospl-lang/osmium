use crate::ast::Expression;

pub struct BinaryOp {
    pub left: Expression,
    pub right: Expression,
    pub kind: BinaryOpType
}

pub enum BinaryOpType {
    Add,
    Subtract,
    Multiply,
    Divide,
    Modulo,
}
