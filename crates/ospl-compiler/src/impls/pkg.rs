use crate::{Compiler, package::Module};

impl Compiler {
    pub fn get_mut_module(&mut self, s: &str) -> Option<&mut Module> {
        return self.packages.modules.get_mut(s);
    }

    pub fn get_module(&mut self, s: &str) -> Option<&Module> {
        return self.packages.modules.get(s);
    }
}
