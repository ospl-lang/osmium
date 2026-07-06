use crate::ast::{Expr, LV, Stmt};

pub enum MacroType {
    Int,
    List(Box<MacroType>),

    Expression,
    Statement,
    Identifier,
    LValue,
}

pub enum MacroValue {
    Int(i64),
    List(Vec<MacroValue>),

    Expression(Expr),
    Statement(Stmt),
    Identifier(String),
    LValue(LV),
}

pub enum MacroExpr {
    Literal(MacroValue),
    Input(String),
}

pub enum MacroStmt {
    Return(MacroExpr)
}

pub struct Macro {
    pub inputs: Vec<MacroType>,
    pub body: Vec<MacroStmt>,
    pub returns: MacroType
}
