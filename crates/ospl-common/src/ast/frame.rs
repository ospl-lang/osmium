use std::collections::HashMap;

use crate::ast::{Store, repr::Type};

#[derive(Debug)]
pub enum ScopeError {
    NotFound {
        id: String,
    }
}
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Default, Debug, PartialEq, Clone)]
pub struct ScopeType {
    names: HashMap<String, Store>,
}

impl ScopeType {
    pub fn get(&self, k: &str) -> Result<&Store, ScopeError> {
        return self.names
            .get(k)
            .ok_or_else(|| ScopeError::NotFound {
                id: k.to_owned()
            })
    }

    pub fn get_mut(&mut self, k: &str) -> Result<&mut Store, ScopeError>{
        return self.names
            .get_mut(k)
            .ok_or_else(|| ScopeError::NotFound {
                id: k.to_owned()
            })
    }
}

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Default, Debug, PartialEq, Clone)]
pub struct Scope {
    names: HashMap<String, Store>,
    nextreg: usize,
}

impl Scope {
    pub fn get(&self, k: &str) -> Result<&Store, ScopeError> {
        return self.names
            .get(k)
            .ok_or_else(|| ScopeError::NotFound {
                id: k.to_owned()
            })
    }

    pub fn get_mut(&mut self, k: &str) -> Result<&mut Store, ScopeError>{
        return self.names
            .get_mut(k)
            .ok_or_else(|| ScopeError::NotFound {
                id: k.to_owned()
            })
    }

    pub fn declare(&mut self, k: String, reg: usize, ty: Type) {
        self.names.insert(k, Store::new(reg, ty));
    }

    pub fn next_pre(&mut self) -> usize {
        let i = self.nextreg;
        self.nextreg += 1;
        return i
    }
}