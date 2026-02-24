use std::collections::HashMap;

#[derive(Debug)]
pub enum EvalError {
    CalledUncallable,
    WrongArgCount {
        expected: usize,
        got: usize
    },
    WrongArgType {
        expected: Type,
        found: Type
    }
}

#[derive(Debug)]
pub enum StmtError {
    Eval(EvalError),
}

#[derive(Debug)]
pub enum CompilerError {
    Stmt(StmtError),
    Eval(EvalError),
}

impl From<EvalError> for StmtError {
    fn from(value: EvalError) -> Self {
        return Self::Eval(value)
    }
}

impl From<EvalError> for CompilerError {
    fn from(value: EvalError) -> Self {
        return Self::Eval(value)
    }
}

impl From<StmtError> for CompilerError {
    fn from(value: StmtError) -> Self {
        return Self::Stmt(value)
    }
}

#[derive(Default, Clone, PartialEq, Eq, Debug)]
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

#[derive(Clone, PartialEq, Eq, Debug)]
pub enum TypeData {
    None,
    Function {
        captures: Scope,
        params: Vec<Type>,
        return_type: Box<Type>
    }
}

impl Default for TypeData {
    fn default() -> Self {
        return Self::None
    }
}

#[derive(Default, Clone, PartialEq, Eq, Debug)]
pub struct Type {
    pub data: TypeData
}

#[derive(Default, Clone, PartialEq, Eq, Debug)]
pub struct Local {
    pub register: usize,
    pub ty: Type
}

#[derive(Default, Clone, PartialEq, Eq, Debug)]
pub struct Scope {
    pub declarations: Declarations
}

impl Scope {
    pub fn declare(&mut self, id: String, data: TypeData) -> usize {
        self.declarations.names.insert(id, Local {
            register: self.declarations.next_register,
            ty: Type { data },
        });

        let i = self.declarations.next_register;
        self.declarations.next_register += 1;
        return i
    }
}