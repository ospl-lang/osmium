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

#[derive(Debug, Clone, PartialEq)]
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
    And,
    Or,
    Xor,

    /* bit-shifts */
    LShift,
    RShift
}

impl BinaryOpType {
    pub fn is_integral(&self) -> bool {
        return self.is_arithmatic()
            || self.is_comparison()
            || self.is_shift()
    }

    pub fn is_floating_point(&self) -> bool {
        return self.is_arithmatic() || self.is_comparison()
    }

    pub fn is_shift(&self) -> bool {
        return matches!(self, Self::LShift | Self::RShift)
    }

    pub fn is_arithmatic(&self) -> bool {
        return matches!(self,
            Self::Add | Self::Subtract | Self::Multiply | Self::Divide |
            Self::Modulo
        )
    }

    pub fn is_comparison(&self) -> bool {
        return matches!(self,
            Self::Equals | Self::NotEquals | Self::Ge | Self::Gt | Self::Lt | Self::Le
        )
    }
}

#[derive(Debug, Clone)]
pub enum UnaryOpType {
    LogicNot,
    Copy,
    Atsign,
    Decrement,
    Increment
}

#[derive(Debug, Clone)]
pub struct UnaryOp {
    pub expr: Expression,
    pub kind: UnaryOpType,
}