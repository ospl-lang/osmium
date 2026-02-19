use std::collections::HashMap;

#[derive(Default)]
pub struct Declarations {
    names: HashMap<String, Local>,
    next_register: usize
}

impl Declarations {
    pub fn next_reg_post(&mut self) -> usize {
        let i = self.next_register;
        self.next_register += 1;
        return i
    }

    pub fn get_reg(&self) -> usize {
        return self.next_register
    }

    pub fn next_reg_pre(&mut self) -> usize {
        self.next_register += 1;
        return self.next_register
    }

    pub fn next_reg(&mut self) {
        self.next_register += 1;
    }

    pub fn get_mut(&mut self, id: &str) -> Option<&mut Local> {
        return self.names.get_mut(id)
    }

    pub fn get(&self, id: &str) -> Option<&Local> {
        return self.names.get(id)
    }
}

pub enum TypeData {
    None,
}

impl Default for TypeData {
    fn default() -> Self {
        return Self::None
    }
}

#[derive(Default)]
pub struct Type {
    pub data: TypeData
}

#[derive(Default)]
pub struct Local {
    pub register: usize,
    pub data: TypeData
}

#[derive(Default)]
pub struct Scope {
    pub declarations: Declarations
}

impl Scope {
    pub fn declare(&mut self, id: String, data: TypeData) -> usize {
        self.declarations.names.insert(id, Local {
            register: self.declarations.next_register,
            data 
        });

        let i = self.declarations.next_register;
        self.declarations.next_register += 1;
        return i
    }
}