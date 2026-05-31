use crate::ast::{Expression, UType};

#[derive(Debug, Clone)]
pub struct Declaration {
    pub name: String,
    pub rhs: Expression,
}

#[derive(Debug, Clone)]
pub struct AliasDeclaration {
    pub name: String,
    pub ty: UType,
}
