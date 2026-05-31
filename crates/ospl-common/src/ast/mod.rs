use std::{cmp::Ordering, collections::HashMap, fmt::{Debug, Display}};
use crate::ast::{decl::{AliasDeclaration, Declaration}, ops::AssignOp};

pub use types::*;

#[derive(Debug, Default, Clone, Copy)]
pub struct Position {
    pub line: usize,
    pub column: usize,
    pub token_num: usize,
    pub ch: usize,
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
    // Define(String, Expression),
    Define(Declaration),
    DefineTypeAlias(AliasDeclaration),
    DefineNominalTypeAlias(AliasDeclaration),
    Assign(LValue, Expression),
    Return(Expression),
    Break,
    Continue,
    If(Expression, Vec<Statement>, Vec<Statement>),
    Loop(Vec<Statement>),
    ReturnScope,
    AssignOp(AssignOp),
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
    UnaryOp(ops::UnaryOp),
    Cast(Expression, UType),

    FFILoad(Expression),
    FFIFunc(LValue, Expression, usize, Vec<usize>),
    FFICall(LValue, Vec<Expression>),

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

    /// Indexes from the start to the end
    Index(Expression, Expression),

    /// Slices from the start to the end
    Slice(Expression, Expression, Expression),
}

#[derive(Debug, Clone)]
pub enum Literal {
    Nul,
    Undefined,
    Int(i64),
    Address(u64),
    Float(f64),
    Bool(bool),
    Str(String),
    Char(char),
    List(UType, Vec<Expression>),
    Function(FunctionValue)
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FunctionType<T> {
    pub args: Vec<T>,
    pub ret: T,
}

impl<T: Debug> Display for FunctionType<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "fn(")?;
        for arg in &self.args {
            write!(f, "{arg:?}")?;
        }

        write!(f, ") -> {:?}", self.ret)?;

        return Ok(())
    }
}

/// A single block of variables.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Scope<T> {
    /// A mapping of names to stack indexes / RelAddrs
    map: HashMap<String, Store<T>>,

    next_id: usize,
}

impl<T> Default for Scope<T> {
    fn default() -> Self {
        return Self {
            map: HashMap::new(),
            next_id: 0
        }
    }
}

impl<T: PartialEq + Eq> PartialOrd for Scope<T> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        if self.map == other.map {
            return Some(Ordering::Equal)
        }

        // do we at least contain the same values
        let mut greater = true;
        for (ak, at) in &other.map {
            if self.map.contains_key(ak) {
                if self.map[ak].get_type() == at.get_type() {
                    continue;  // live to see another day
                } else { greater = false }
            } else { greater = false }
        }

        if greater {
            return Some(Ordering::Greater)
        }

        return Some(Ordering::Less)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Store<T> {
    address: Option<usize>,
    typ: T,
}

impl<T> Store<T> {
    pub fn get_address(&self) -> Option<usize> {
        return self.address
    }

    pub fn get_type(&self) -> &T {
        return &self.typ
    }
}

impl<T: Clone> Scope<T> {
    pub fn declare(&mut self, key: String, address: usize, ty: T) {
        self.map.insert(key, Store {
            address: Some(address),
            typ: ty
        });
    }

    pub fn direct_declare(&mut self, key: String, address: Option<usize>, ty: T) {
        self.map.insert(key, Store {
            address: address,
            typ: ty
        });
    }

    pub fn jump(&mut self, to: usize) {
        self.next_id = to;
    }

    pub fn tell(&self) -> usize {
        return self.next_id;
    }

    pub fn declare_non_addressable(&mut self, key: String, ty: T) {
        self.map.insert(key, Store {
            address: None,
            typ: ty
        });
    }

    pub fn delete(&mut self, k: &str) {
        let Some(_) = self.map.remove(k)
        else {
            panic!("TODO unwrap - tried to delete key {k} that doesn't exist from a scope type")
        };
    }

    pub fn has(&self, key: &str) -> bool {
        return self.map.contains_key(key)
    }

    /// This function may be removed in the future! DO NOT USE unless you
    /// REALLY NEED IT
    pub fn get_inner(&self) -> &HashMap<String, Store<T>> {
        return &self.map
    }

    pub fn get_combined(&self, k: &str) -> Option<(usize, &T)> {
        let x = self.map.get(k)?;
        return Some((x.address?, &x.typ))
    }

    pub fn get_combined_copy(&self, k: &str) -> Option<(usize, T)> {
        let x = self.map.get(k)?;
        return Some((x.address?, x.typ.clone()))
    }

    pub fn get_combined_with_nonaddressable(&self, k: &str) -> Option<(Option<usize>, &T)> {
        let x = self.map.get(k)?;
        return Some((x.address, &x.typ))
    }

    pub fn next_post(&mut self) -> usize {
        let i = self.next_id;
        self.next_id += 1;
        return i
    }
}

#[derive(Debug, Clone)]
pub struct FunctionValue {
    pub ftype: FunctionType<UType>,

    /// Argument names, take the index in the array of the target argument to
    /// and index into the function type's array to get the value
    pub args: Vec<ArgNaming>,

    pub captures: Vec<String>,

    pub block: Vec<Statement>,
}

#[derive(Debug, Clone)]
pub struct ArgNaming {
    pub name: String,
}

pub mod frame;
pub mod ops;
pub mod spanning;
pub mod decl;
mod types;