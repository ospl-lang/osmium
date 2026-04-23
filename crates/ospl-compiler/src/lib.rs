use std::collections::HashMap;
use ospl_common::ast::{class::instance::Entity, module::VScope, repr::Type};
use crate::base::{CompilerError, Res};

pub mod base;
mod impls;

#[cfg(test)]
mod tests;

#[derive(Debug, Clone)]
pub struct LocalStore {
    reg: usize,
    ty: Type,
}

impl Entity for LocalStore {
    fn get_index(&self) -> usize {
        return self.reg
    }

    fn get_type(&self) -> &Type {
        return &self.ty
    }
}

#[derive(Default, Clone)]
pub struct Scope {
    names: HashMap<String, LocalStore>,
    nextreg: usize,
}

impl Scope {
    pub fn get_local(&self, k: &str) -> Res<&LocalStore> {
        return self.names
            .get(k)
            .ok_or_else(|| CompilerError::NotFoundInScope {
                id: k.to_owned()
            })
    }

    pub fn get_mut_local(&mut self, k: &str) -> Res<&mut LocalStore> {
        return self.names
            .get_mut(k)
            .ok_or_else(|| CompilerError::NotFoundInScope {
                id: k.to_owned()
            })
    }

    pub fn declare(&mut self, k: String, reg: usize, ty: Type) {
        self.names.insert(k, LocalStore {
            reg, ty
        });
    }

    pub fn next_pre(&mut self) -> usize {
        let i = self.nextreg;
        self.nextreg += 1;
        return i
    }
}

pub struct Compiler {
    scopes: Vec<Scope>,
    vroot: VScope,
}

impl Compiler {
    pub fn new() -> Self {
        return Self {
            scopes: vec![
                Scope::default()
            ],
            vroot: VScope::default()
        }
    }

    #[allow(unused)]
    fn top(&self) -> Res<&Scope> {
        return self.scopes
            .last()
            .ok_or_else(|| CompilerError::AtLeastOneScopeRequired)
    }

    fn top_mut(&mut self) -> Res<&mut Scope> {
        return self.scopes
            .last_mut()
            .ok_or_else(|| CompilerError::AtLeastOneScopeRequired)
    }

    fn new_scope_parent(&mut self) -> Res<()> {
        let sc = self.top()?;
        self.scopes.push(sc.clone());

        return Ok(())
    }

    fn new_scope_isolated(&mut self) -> Res<()> {
        self.scopes.push(Scope::default());

        return Ok(())
    }

    fn pop_scope(&mut self) -> Res<()> {
        self.scopes.pop()
            .unwrap();  // FIX-UNWRAP: change this to error handling

        return Ok(())
    }
}