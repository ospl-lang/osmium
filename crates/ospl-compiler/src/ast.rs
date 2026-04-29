use crate::{RelativeVarID, Type};

pub struct Statement {
    pub inner: Box<Stmt>
}

pub enum Stmt {
    Define(String, Type, Expression),
    Return(Expression),
    ReturnScope,
}

pub struct Expression {
    pub inner: Box<Expr>
}

pub enum Expr {
    Literal(Literal),
    Call(Expression, Vec<Expression>),
    LValue(LValue),
}

pub struct LValue {
    pub inner: Box<LV>
}

pub enum LV {
    Variable(String),
    Property(LValue, String),
}

pub enum Literal {
    Int(i64),
    Function(FunctionValue)
}

#[derive(Clone)]
pub struct FunctionType {
    pub captures: Vec<RelativeVarID>,
    pub args: Vec<Type>,
    pub ret: Type,
}

impl PartialEq for FunctionType {
    fn eq(&self, other: &Self) -> bool {
        // just ignore captures
        return (self.args == other.args)
            && self.ret == other.ret
    }
}

pub struct FunctionValue {
    pub ftype: FunctionType,
    pub block: Vec<Statement>,
}
