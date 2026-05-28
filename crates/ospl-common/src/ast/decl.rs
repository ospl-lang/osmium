use crate::ast::{Expression, UType};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Visibility {
    Public,
    Private,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Constness {
    Mut,
    Const,
}

#[derive(Debug, Clone)]
pub struct Declaration {
    pub meta: DeclarationMeta,
    pub name: String,
    pub rhs: Expression,
}

#[derive(Debug, Clone)]
pub struct AliasDeclaration {
    pub meta: DeclarationMeta,
    pub name: String,
    pub ty: UType,
}

#[derive(Debug, Clone)]
/// Serves as metadata for the toolchain rather than real compiler data
pub struct DeclarationMeta {
    pub visibility: Visibility,
    pub constness: Constness,
}