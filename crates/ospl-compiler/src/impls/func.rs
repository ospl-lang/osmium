use ospl_common::inst::optimized::{Inst, InstBuilder};

use crate::{Compiler, EvalResult, Type, ast::FunctionValue};

impl Compiler {
    pub fn fn_literal(
        &mut self,
        func: &FunctionValue,
        ob: &mut Vec<Inst>
    ) -> EvalResult
    {
        let mut insts = Vec::<Inst>::new();
        for stmt in &func.block {
            self.compile_stmt(stmt, &mut insts);
        };

        let inst = InstBuilder::new()
            .child(insts)
            .indexes(&func.ftype.captures)
            .build();

        ob.push(inst);
        return EvalResult {
            address: self.next_var(),
            ty: Type::Function(Box::new(func.ftype.clone()))
        }
    }
}