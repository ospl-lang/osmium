pub struct Statement {
    pub inner: Box<Stmt>
}

impl Statement {
    pub fn test(s: Stmt) -> Self {
        return Self {
            inner: Box::new(s)
        }
    }
}

pub enum Stmt {
    Define(String, Expression),
    Return(Expression),
    ReturnScope,
}

pub struct Expression {
    pub inner: Box<Expr>
}

impl Expression {
    pub fn test(s: Expr) -> Self {
        return Self {
            inner: Box::new(s)
        }
    }
}

pub enum Expr {
    Literal(Literal),
    Call(Expression, Vec<Expression>),
    LValue(LValue),
}

pub struct LValue {
    pub inner: Box<LV>
}

impl LValue {
    pub fn test(s: LV) -> Self {
        return Self {
            inner: Box::new(s)
        }
    }
}

pub enum LV {
    Variable(String),
    Property(LValue, String),
}

pub enum Literal {
    Int(i64),
    Function(FunctionValue)
}

#[derive(Debug, Clone)]
pub struct FunctionType {
    /// Relative var ID
    pub captures: Vec<usize>,
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

pub mod frame;