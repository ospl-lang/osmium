use ospl_common::inst::optimized::{Inst, InstBuilder, Opc};

pub enum Control {
    Default,
    Return(usize),
    ReturnScope,
}

use crate::{Compiler, Res, ast::{Statement, Stmt}};

impl Compiler {
    pub fn compile_stmt(
        &mut self,
        s: &Statement,
        ob: &mut Vec<Inst>
    ) -> Res<Control>
    {
        match &*s.inner {
            // TODO: check type
            Stmt::Define(var, init) => {
                let eval = self.eval(init, ob)?;

                self.stack.top_mut().declare(var.to_string(), eval.address, eval.ty);
            },
            Stmt::ReturnScope => {
                let inst = InstBuilder::new()
                    .opcode(Opc::RetScope)
                    .build();
                
                ob.push(inst);
                return Ok(Control::ReturnScope)
            },
            Stmt::Return(e) => {
                let eval = self.eval(e, ob)?;
                return Ok(Control::Return(eval.address))
            },
            _ => unimplemented!("TODO - impl the other stmts")
        }

        return Ok(Control::Default)
    }

    pub fn compile_block(
        &mut self,
        s: &[Statement],
        ob: &mut Vec<Inst>
    ) -> Res<()>
    {
        for stmt in s {
            self.compile_stmt(stmt, ob)?;
        }

        return Ok(())
    }
}