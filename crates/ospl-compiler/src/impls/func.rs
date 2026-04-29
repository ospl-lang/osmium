use ospl_common::{ast::Expression, inst::optimized::{Inst, InstBuilder, Opc}};

use crate::{Compiler, EvalResult, Res, Type, ast::FunctionValue};

impl Compiler {
    pub fn fn_literal(
        &mut self,
        func: &FunctionValue,
        ob: &mut Vec<Inst>
    ) -> Res<EvalResult>
    {
        let mut insts = Vec::<Inst>::new();
        for stmt in &func.block {
            self.compile_stmt(stmt, &mut insts)?;
        };

        let inst = InstBuilder::new()
            .child(insts)
            .indexes(&func.ftype.captures)
            .build();

        ob.push(inst);
        return Ok(EvalResult {
            address: self.next_var(),
            ty: Type::Function(Box::new(func.ftype.clone()))
        })
    }

    pub fn do_call(
        &mut self,
        func: &Expression,
        args: &[Expression],
        ob: &mut Vec<Inst>,
    ) -> Res<EvalResult>
    {
        let f = self.eval(func, ob)?;
        let Type::Function(func) = f.ty
            else { unimplemented!() };  // FIXME: use result

        // TYPECHECKING...
        if func.args.len() != args.len() {
            // wrong number of args
            // FIXME: actually throw error
        }
        
        let mut new_args = Vec::new();
        for arg in args {
            let eval = self.eval(arg, ob)?;
            new_args.push(eval.address);
        }

        ob.push(InstBuilder::new()
            .opcode(Opc::Call)
            .index(f.address)
            .indexes(&new_args)
            .build());

        return Ok(EvalResult {
            address: self.next_var(),
            ty: func.ret.clone()
        })
    }
}