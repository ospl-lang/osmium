use ospl_common::inst::optimized::Inst;

pub enum Control {
    Default,
    Return(usize),
    ReturnScope,
}

use crate::{Compiler, ast::{Statement, Stmt}};

impl Compiler {
    pub fn compile_stmt(
        &mut self,
        s: &Statement,
        ob: &mut Vec<Inst>
    ) -> Control
    {
        match &*s.inner {
            // TODO: check type
            Stmt::Define(var, init) => {
                let eval = self.eval(init, ob);

                self.stack.top_mut().declare(var.to_string(), eval.address, eval.ty);
            },
            Stmt::ReturnScope => return Control::ReturnScope,
            Stmt::Return(e) => {
                let eval = self.eval(e, ob);
                return Control::Return(eval.address)
            }
        }

        return Control::Default
    }
}