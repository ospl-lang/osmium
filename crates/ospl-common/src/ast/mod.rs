use std::collections::HashMap;

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
    BinaryOp(ops::BinaryOp),
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

#[derive(Debug, Clone, PartialEq)]
pub enum Type {
    Nul, Undefined,
    Int,
    Scope(Scope),
    Function(Box<FunctionType>)
}

/// A single block of variables.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Scope {
    /// A mapping of names to stack indexes / RelAddrs
    map: HashMap<String, usize>,
    
    /// The types of each RelAddrs, starting from 0
    types: HashMap<usize, Type>,

    next_id: usize,
}

impl Scope {
    pub fn declare(&mut self, key: String, address: usize, ty: Type) {
        self.map.insert(key, address);
        self.types.insert(address, ty);
    }

    pub fn get_map(&mut self) -> &HashMap<String, usize> {
        return &self.map
    }

    pub fn get_combined(&self, k: &str) -> Option<(usize, &Type)> {
        let x = self.map.get(k)?;
        let t = self.types.get(x)?;
        return Some((*x, t))
    }

    pub fn get_types(&mut self) -> &HashMap<usize, Type> {
        return &self.types
    }

    pub fn next_post(&mut self) -> usize {
        let i = self.next_id;
        self.next_id += 1;
        return i
    }
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
pub mod ops;