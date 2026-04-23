use std::{collections::HashMap, fmt::Debug};

use crate::ast::repr::Type;

#[derive(Debug, PartialEq, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct InstanceType {
    pub values: HashMap<String, InstanceItem>
}

#[derive(Debug, PartialEq, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct InstanceItem {
    pub ty: Box<Type>,
    pub index: usize,
}

impl Entity for InstanceItem {
    fn get_index(&self) -> usize {
        return self.index
    }

    fn get_type(&self) -> &Type {
        return &self.ty
    }
}

pub trait Entity: Debug {
    fn get_type(&self) -> &Type;
    fn get_index(&self) -> usize;
}
