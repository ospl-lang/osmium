#[derive(Debug, PartialEq, Eq)]
pub enum Op {
    Add,
    Sub,
    Mul,
    Div,
    Mod,
}

#[derive(Debug, PartialEq)]
pub enum Expr {
    StaticLiteral(StaticValue),
    BinaryOp {
        op: Op,
        lhs: Box<Expr>,
        rhs: Box<Expr>,
    },
    Call(LValue, Vec<Expr>),
    LValue(LValue)
}

#[derive(Debug, PartialEq, Clone)]
pub enum StaticValue {
    Int(i64),
    Float(f64),
    Nul
}

#[derive(Debug, PartialEq, Eq)]
pub enum LValue {
    Var(String),
}

#[derive(Debug, PartialEq)]
pub enum Stmt {
    Declare {
        lhs: LValue,
        rhs: Option<Expr>,
    },
    Assign {
        lhs: LValue,
        rhs: Expr,
    },
}
