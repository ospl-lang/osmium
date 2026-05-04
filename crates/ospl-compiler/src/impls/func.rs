use ospl_common::{ast::Expression, inst::optimized::{Inst, InstBuilder, Opc}};

use crate::{CompErr, Compiler, EvalResult, Res, Type, ast::FunctionValue, impls::stmt::Control};

impl Compiler {
    pub fn fn_literal(
        &mut self,
        func: &FunctionValue,
        ob: &mut Vec<Inst>
    ) -> Res<EvalResult>
    {
        self.stack.push();

        // add the aregs
        let mut arg_indexes = Vec::new();
        for (arg, typ) in func.args.iter().zip(func.ftype.args.iter()) {
            let variable = self.next_var();

            self.stack.top_mut().declare(arg.clone(), variable, typ.clone());
            arg_indexes.push(variable);
        }

        // add the closures
        let mut capture_indexes = Vec::new();
        for capture in &func.captures {
            let variable = self.next_var();

            let (u, t) = {
                let idx = self.stack.scopes.len() - 2;
                let scope = self.stack.scopes.get(idx)
                    .ok_or_else(|| CompErr::NoScopeToCapture)?;

                scope.get_combined_copy(&capture)
                    .ok_or_else(|| CompErr::NotFoundInScope { needed: capture.clone() })?
            };
            self.stack.top_mut().declare(capture.clone(), variable, t);

            capture_indexes.push(u);
        }

        let mut insts = Vec::<Inst>::new();
        let mut has_a_return = false;
        for stmt in &func.block {
            match self.compile_stmt(stmt, &mut insts)? {
                Control::Return(_) => has_a_return = true,
                _ => {}
            }
        };

        // insert a return at the end if we don't have one
        if !has_a_return {
            let v = self.next_var();

            insts.push(InstBuilder::new()
                .opcode(Opc::PushLiteral)
                .value(ospl_common::inst::RuntimeValue::Nul)
                .build());

            insts.push(InstBuilder::new()
                .opcode(Opc::Ret)
                .index(v)
                .build());
        }

        let inst = InstBuilder::new()
            .opcode(Opc::PushFunction)
            .child(insts)
            .indexes(&capture_indexes)
            .build();

        ob.push(inst);

        self.stack.pop();

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
            return Err(CompErr::WrongArgCount {
                expected: func.args.len(),
                got: args.len()
            })
        }

        let mut new_args = Vec::new();
        for (arg, func_arg) in args.iter().zip(&func.args) {
            let eval = self.eval(arg, ob)?;
            if eval.ty != *func_arg {
                return Err(CompErr::MismatchedTypes {
                    expected: func_arg.clone(),
                    got: eval.ty
                })
            }
            new_args.push(eval.address);
        }
        let i = InstBuilder::new()
            .opcode(Opc::Call)
            .index(f.address)  // right here
            .indexes(&new_args)
            .build();

        ob.push(i);

        return Ok(EvalResult {
            address: self.next_var(),
            ty: func.ret.clone()
        })
    }
}