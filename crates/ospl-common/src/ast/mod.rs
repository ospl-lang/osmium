use crate::ast::repr::Type;

pub mod repr;
pub mod module;
pub mod frame;

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, PartialEq, Clone)]
pub struct Store {
    /// Don't use this to retrieve stuff.
    /// Only to generate code!
    reg: usize,

    ty: Type,
}

impl Store {
    /// Don't use this to retrieve stuff.
    /// Only to generate code!
    pub fn reg(&self) -> usize {
        return self.reg
    }

    pub fn get_type(&self) -> &Type {
        return &self.ty
    }

    pub fn new(reg: usize, ty: Type) -> Self {
        return Self { reg, ty }
    }
}

pub struct Expression<'a> {
    pub position: Position<'a>,
    pub inner: repr::Expr,
}

pub struct Statement<'a> {
    pub position: Position<'a>,
    pub inner: repr::Stmt,
}

pub struct Position<'a> {
    pub line: usize,
    pub column: usize,
    pub character: usize,
    pub file: &'a str,
}
