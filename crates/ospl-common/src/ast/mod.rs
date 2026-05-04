use std::{collections::HashMap, fmt::Display};

#[derive(Debug, Default, Clone, Copy)]
pub struct Position {
    pub line: usize,
    pub column: usize,
    pub ch: usize
}

impl Display for Position {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        return write!(f, "{}:{} (char #{})", self.line, self.column, self.ch)
    }
}

impl Position {
    pub fn next_line(&mut self) {
        self.line += 1;
        self.column = 0;
    }

    pub fn next_column(&mut self) {
        self.column += 1;
    }
}

#[derive(Debug, Clone)]
pub struct Statement {
    pub inner: Box<Stmt>,
    pub at: Position,
}

impl Statement {
    pub fn test(s: Stmt) -> Self {
        return Self {
            inner: Box::new(s),
            at: Position::default()
        }
    }
}

#[derive(Debug, Clone)]
pub enum Stmt {
    Expr(Expression),
    Define(String, Expression),
    Assign(LValue, Expression),
    Return(Expression),
    Break,
    Continue,
    If(Expression, Vec<Statement>, Vec<Statement>),
    Loop(Vec<Statement>),
    ReturnScope,
}

#[derive(Debug, Clone)]
pub struct Expression {
    pub inner: Box<Expr>,
    pub at: Position,
}

impl Expression {
    pub fn test(s: Expr) -> Self {
        return Self {
            inner: Box::new(s),
            at: Position::default()
        }
    }
}

#[derive(Debug, Clone)]
pub enum Expr {
    Literal(Literal),
    Call(Expression, Vec<Expression>),
    BinaryOp(ops::BinaryOp),
    LValue(LValue),
}

#[derive(Debug, Clone)]
pub struct LValue {
    pub inner: Box<LV>,
    pub at: Position
}

impl LValue {
    pub fn test(s: LV) -> Self {
        return Self {
            inner: Box::new(s),
            at: Position::default()
        }
    }
}

#[derive(Debug, Clone)]
pub enum LV {
    Variable(String),
    Property(LValue, String),
}

#[derive(Debug, Clone)]
pub enum Literal {
    Int(i64),
    Float(f64),
    Bool(bool),
    Str(String),
    Function(FunctionValue)
}

#[derive(Debug, Clone)]
pub struct FunctionType {
    /// Relative var ID
    // pub captures: Vec<usize>,
    pub args: Vec<Type>,
    pub ret: Type,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Type {
    Nul, Undefined,
    Int, Float, Str, Bool, List,
    Scope(Scope),
    Function(Box<FunctionType>),
    TypeOfVar(String),
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
        // println!("DECLARE: {:?}", self);
    }

    pub fn get_map(&mut self) -> &HashMap<String, usize> {
        return &self.map
    }

    pub fn get_combined(&self, k: &str) -> Option<(usize, &Type)> {
        let x = self.map.get(k)?;
        let t = self.types.get(x)?;
        return Some((*x, t))
    }

    pub fn get_combined_copy(&self, k: &str) -> Option<(usize, Type)> {
        let x = self.map.get(k)?;
        let t = self.types.get(x)?.clone();
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

#[derive(Debug, Clone)]
pub struct FunctionValue {
    pub ftype: FunctionType,

    /// Argument names, take the index in the array of the target argument to
    /// and index into the function type's array to get the value
    pub args: Vec<String>,

    /// Are derieved by the literal generator (in the compiler)
    pub captures: Vec<String>,

    pub block: Vec<Statement>,
}

pub mod frame;
pub mod ops;
pub mod spanning;