use ospl_common::{ast::{Expression, Statement}, inst::optimized::{InstBuilder, Opc}};

use crate::{Compiler, Res};

impl<'a> Compiler<'a> {
    pub fn compile_if_stmt(
        &mut self,
        left: &Expression,
        yes: &[Statement],
        no: &[Statement],
    ) -> Res<()>
    {
        let l = self.eval(left)?;

        // yes
        self.stack.push_parental();
        let yes_start = self.insts.len();
        self.compile_block(yes)?;
        let yes_end = self.insts.len();
        self.stack.pop();

        // no
        self.stack.push_parental();
        let no_start = self.insts.len();
        self.compile_block(no)?;
        let no_end = self.insts.len();
        self.stack.pop();

        let i = InstBuilder::new()
            .opcode(Opc::If)
            .index(l.address)
            .index(yes_start)
            .index(yes_end)  // same as no_start
            .index(no_start)
            .index(no_end)
            .build();
    
        self.insts.push(i);
        return Ok(())
    }

    pub fn compile_loop_stmt(
        &mut self,
        inner: &[Statement],
    ) -> Res<()>
    {
        self.stack.push_parental();
        let loop_start = self.insts.len();
        self.compile_block(inner)?;
        let loop_end = self.insts.len();

        self.stack.pop();
        let i = InstBuilder::new()
            .opcode(Opc::Loop)
            .index(loop_start)
            .index(loop_end)
            .build();

        self.insts.push(i);
        return Ok(())
    }
}
