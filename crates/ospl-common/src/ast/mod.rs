use crate::ast::{
    decl::{AliasDeclaration, Declaration},
    ops::AssignOp,
};
use std::{
    collections::{BTreeMap, HashMap},
    fmt::{Debug, Display},
    hash::{Hash, Hasher},
    sync::Arc,
};

pub use types::*;

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct Position {
    pub line: usize,
    pub column: usize,
    pub token_num: usize,
    pub ch: usize,
}

impl PartialOrd for Position {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        return self.ch.partial_cmp(&other.ch);
    }
}

impl Display for Position {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        return write!(f, "{}:{} (char #{})", self.line, self.column, self.ch);
    }
}

impl Position {
    pub fn next_line(&mut self) {
        self.line += 1;
        self.column = 1;
    }

    pub fn next_column(&mut self) {
        self.column += 1;
    }
}

#[derive(Debug, Clone)]
pub struct Statement {
    pub inner: Box<Stmt>,
    pub at: Position,
    pub notes: String,
    pub file: Arc<String>,
}

impl Statement {
    pub fn test(s: Stmt) -> Self {
        return Self {
            inner: Box::new(s),
            at: Position::default(),
            notes: String::new(),
            file: Arc::new(String::new()),
        };
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
    TypeIf {
        lhs: Expression,
        typ: UType,
        id: String,
        yes: Vec<Statement>,
        no: Vec<Statement>
    },
    Loop(Vec<Statement>),
    ReturnScope,
    AssignOp(AssignOp),
}

#[derive(Debug, Clone)]
pub struct Expression {
    pub inner: Box<Expr>,
    pub at: Position,
    pub file: Arc<String>,
}

impl Expression {
    pub fn test(s: Expr) -> Self {
        return Self {
            inner: Box::new(s),
            at: Position::default(),
            file: Arc::new(String::from("test")),
        };
    }
}

#[derive(Debug, Clone)]
pub enum Expr {
    Literal(Literal),
    Call {
        left: Expression,
        args: Vec<Expression>,
        named_args: BTreeMap<String, Expression>,
    },
    BinaryOp(ops::BinaryOp),
    UnaryOp(ops::UnaryOp),
    Cast(Expression, UType),
    Apply(Expression, Vec<(UType, UType)>),

    Block(Vec<Statement>),

    FFILoad(Expression),
    FFIFunc(LValue, Expression, usize, Vec<usize>),
    FFICall(LValue, Vec<Expression>),

    LValue(LValue),
}

#[derive(Debug, Clone)]
pub struct LValue {
    pub inner: Box<LV>,
    pub at: Position,
    pub file: Arc<String>,
}

impl LValue {
    pub fn test(s: LV) -> Self {
        return Self {
            inner: Box::new(s),
            at: Position::default(),
            file: Arc::new(String::new()),
        };
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
    Function(FunctionValue),
}

#[derive(Debug, Clone, Eq)]
pub struct FunctionType<T> {
    pub args: Vec<T>,
    pub named_args: BTreeMap<String, T>,
    pub ret: T,
}

// this manual implementation is here incase we need it.
// and also I wanted to make the requirements more explicit.
impl<T: PartialEq> PartialEq for FunctionType<T> {
    fn eq(&self, other: &Self) -> bool {
        let arity_good =
            self.args.len() == other.args.len() && self.named_args.len() == other.named_args.len();

        let args_good = self
            .args
            .iter()
            .zip(other.args.iter())
            .all(|(a, b)| *a == *b);

        let named_args_good = self
            .named_args
            .iter()
            .zip(other.named_args.iter())
            .all(|(a, b)| a == b);

        let ret_good = self.ret == other.ret;

        return arity_good && args_good && named_args_good && ret_good;
    }
}

impl<T: Hash> Hash for FunctionType<T> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.args.hash(state);
        for thing in &self.named_args {
            thing.hash(state);
        }

        self.ret.hash(state);
    }
}

impl<T: Debug> Display for FunctionType<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "fn(")?;
        for arg in &self.args {
            write!(f, "{arg:?}")?;
        }

        write!(f, ") -> {:?}", self.ret)?;

        return Ok(());
    }
}

/// A single block of variables.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Scope<T> {
    map: HashMap<String, Entry<T>>,

    next_id: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Entry<T> {
    Runtime(RStore<T>),
    Alias(AStore<T>),
}

impl<T> Entry<T> {
    pub fn get_type(&self) -> Option<&T> {
        let t = match self {
            Entry::Alias(a) => &a.typ,
            Entry::Runtime(rt) => &rt.typ,
            // _ => return None
        };

        Some(t)
    }

    pub fn map_type_into<I>(self, f: impl Fn(T) -> I) -> Entry<I> {
        return match self {
            Self::Alias(a) => Entry::Alias(AStore { typ: f(a.typ) }),
            Self::Runtime(r) => Entry::Runtime(RStore {
                address: r.address,
                typ: f(r.typ),
            }),
        };
    }

    pub fn map_type_into_resultant<I, E>(
        self,
        f: impl Fn(T) -> Result<I, E>,
    ) -> Result<Entry<I>, E> {
        return Ok(match self {
            Self::Alias(a) => Entry::Alias(AStore { typ: f(a.typ)? }),
            Self::Runtime(r) => Entry::Runtime(RStore {
                address: r.address,
                typ: f(r.typ)?,
            }),
        });
    }
}

impl<T: Hash> Hash for Entry<T> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        match self {
            Self::Runtime(r) => r.hash(state),
            Self::Alias(a) => a.hash(state),
        }
    }
}

impl<T: Hash> Hash for Scope<T> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.next_id.hash(state);

        // Hash the map in a deterministic way
        // HashMap iteration order is NOT stable, so you must normalize it
        let mut entries: Vec<_> = self.map.iter().collect();
        entries.sort_by(|a, b| a.0.cmp(b.0));

        for (k, v) in entries {
            k.hash(state);
            v.hash(state);
        }
    }
}

impl<T> Default for Scope<T> {
    fn default() -> Self {
        return Self {
            map: HashMap::new(),
            next_id: 0,
        };
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct RStore<T> {
    pub address: usize,
    pub typ: T,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct AStore<T> {
    pub typ: T,
}

impl<T> RStore<T> {
    pub fn get_address(&self) -> usize {
        return self.address;
    }

    pub fn get_type(&self) -> &T {
        return &self.typ;
    }

    pub fn into_type(self) -> T {
        return self.typ;
    }
}

impl<T: Clone> Scope<T> {
    pub fn direct_declare(&mut self, key: String, e: Entry<T>) {
        self.map.insert(key, e);
    }

    pub fn jump(&mut self, to: usize) {
        self.next_id = to;
    }

    pub fn tell(&self) -> usize {
        return self.next_id;
    }

    pub fn delete(&mut self, k: &str) {
        let Some(_) = self.map.remove(k) else {
            panic!("TODO unwrap - tried to delete key {k} that doesn't exist from a scope type")
        };
    }

    pub fn has(&self, key: &str) -> bool {
        return self.map.contains_key(key);
    }

    /// This function may be removed in the future! DO NOT USE unless you
    /// REALLY NEED IT
    pub fn get_inner(&self) -> &HashMap<String, Entry<T>> {
        return &self.map;
    }

    /// Gets a runtime value
    pub fn get_runtime(&self, t: &str) -> Option<&RStore<T>> {
        return match self.get_inner().get(t)? {
            Entry::Runtime(r) => Some(r),
            _ => None,
        };
    }

    /// Gets an alias
    pub fn get_alias(&self, t: &str) -> Option<&AStore<T>> {
        return match self.get_inner().get(t)? {
            Entry::Alias(a) => Some(a),
            _ => None,
        };
    }

    pub fn get_entry(&self, t: &str) -> Option<&Entry<T>> {
        return self.get_inner().get(t);
    }

    pub fn get_entry_type(&self, t: &str) -> Option<&T> {
        return self.get_inner().get(t)?.get_type();
    }

    pub fn into_inner(self) -> HashMap<String, Entry<T>> {
        return self.map;
    }

    pub fn next_post(&mut self) -> usize {
        let i = self.next_id;
        self.next_id += 1;
        return i;
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

pub mod decl;
pub mod frame;
pub mod ops;
pub mod spanning;
pub mod types;
