use std::{collections::HashMap, fmt::Display};
use crate::ast::{decl::{AliasDeclaration, Declaration, Visibility}, ops::AssignOp, types::{FunctionType, UType}};
pub use types::Type;

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
    Call {
        func: Expression,
        args: Vec<Expression>,
        generics: Vec<UType>,
    },
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

/// A single block of variables.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Scope<T> {
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Store<T> {
    pub address: StoreAddress,
    pub typ: T,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StoreAddress {
    None,
    Generic(usize),
    With(usize),
}

impl<T: Clone> Scope<T> {
    pub fn declare(&mut self, key: String, address: usize, ty: T) {
        self.map.insert(key, Store {
            address: StoreAddress::With(address),
            typ: ty
        });
    }

    pub fn direct_dcl(&mut self, key: String, address: StoreAddress, ty: T) {
        self.map.insert(key, Store {
            address,
            typ: ty
        });
    }

    pub fn delete(&mut self, k: &str) {
        let Some(_) = self.map.remove(k)
        else {
            panic!("TODO unwrap - tried to delete key {k} that doesn't exist from a scope type")
        };
    }

    /* #[allow(private_interfaces)]
    pub fn keys(&self) -> std::collections::hash_map::Keys<'_, String, Store<T>> {
        return self.map.keys()
    } */

    pub fn keys_cloned(&self) -> Vec<String> {
        return self.map.keys().cloned().collect()
    }

    pub fn has(&self, key: &str) -> bool {
        return self.map.contains_key(key)
    }

    pub fn get_combined(&self, k: &str) -> Option<(usize, &T)> {
        let x = self.map.get(k)?;
        if let StoreAddress::With(w) = &x.address {
            return Some((*w, &x.typ))
        } else { return None }
    }

    pub fn get_store(&self, k: &str) -> Option<&Store<T>> {
        return self.map.get(k)
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

    /// Names of generics
    pub generics: Vec<String>,

    pub captures: Vec<String>,

    pub block: Vec<Statement>,
}

#[derive(Debug, Clone)]
pub struct ArgNaming {
    pub name: String,
    pub privacy: Visibility,
}

pub mod frame;
pub mod ops;
pub mod spanning;
pub mod decl;
pub mod types;