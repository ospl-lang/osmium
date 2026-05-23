use ospl_common::{ast::{Expression, Statement}, inst::optimized::{Inst, InstBuilder, Opc}};

use crate::{Compiler, Res};

impl Compiler {
    pub fn compile_if_stmt(
        &mut self,
        left: &Expression,
        yes: &[Statement],
        no: &[Statement],
        ob: &mut Vec<Inst>
    ) -> Res<()>
    {
        let l = self.eval(left, ob)?;

        let mut ob_yes = Vec::new();
        self.stack.push_parental();
        self.compile_block(yes, &mut ob_yes)?;
        self.stack.pop();

        let mut ob_no = Vec::new();
        self.stack.push_parental();
        self.compile_block(no, &mut ob_no)?;
        self.stack.pop();

        let i = InstBuilder::new()
            .opcode(Opc::If)
            .index(l.address)
            .child(ob_yes)
            .child(ob_no)
            .build();
    
        ob.push(i);
        return Ok(())
    }

    pub fn compile_loop_stmt(
        &mut self,
        inner: &[Statement],
        ob: &mut Vec<Inst>
    ) -> Res<()>
    {
        self.stack.push_parental();
        let mut loop_ob = Vec::new();
        for s in inner {
            self.compile_stmt(s, &mut loop_ob)?;
        }

        let i = InstBuilder::new()
            .opcode(Opc::Loop)
            .child(loop_ob)
            .build();

        ob.push(i);
        self.stack.pop();
        return Ok(())
    }
}