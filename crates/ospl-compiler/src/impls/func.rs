use ospl_common::{ast::{Expression, spanning::IdkWhere}, inst::optimized::{Inst, InstBuilder, Opc}};

use crate::{CE, CEData, Compiler, EvalResult, Res, Type, ast::FunctionValue, impls::stmt::Control};

impl Compiler {
    pub fn fn_literal(
        &mut self,
        func: &FunctionValue,
        ob: &mut Vec<Inst>
    ) -> Res<EvalResult>
    {
        self.stack.push();

        // for inference of the return type if we're returning a scope
        let mut real_next_index = 0;

        // add the aregs
        let mut arg_indexes = Vec::new();
        for (arg, typ) in func.args.iter().zip(func.ftype.args.iter()) {
            let variable = self.next_var();

            self.stack.top_mut().declare(arg.clone(), variable, typ.clone());
            arg_indexes.push(variable);

            // don't declare args in the inferred type
            real_next_index += 1;
        }

        // add the captures
        let mut capture_indexes = Vec::new();
        for capture in &func.captures {
            let variable = self.next_var();

            let (u, t) = {
                let idx = self.stack.scopes.len() - 2;
                let scope = self.stack.scopes.get(idx)
                    .ok_or_else(|| CE {
                        at: Box::new(IdkWhere),
                        msg: Some("perhaps you failed preschool?"),
                        error: CEData::NoScopeToCapture,
                    })?;

                scope.get_combined_copy(&capture)
                    .ok_or_else(|| CE {
                        at: Box::new(IdkWhere),
                        msg: Some("perhaps you meant to chain the capture across multiple scopes?"),
                        error: CEData::NotFoundInScope { needed: capture.clone() }
                    })?
            };
            self.stack.top_mut().declare(capture.clone(), variable, t);

            capture_indexes.push(u);
            real_next_index += 1;

            // don't declare captures in the inferred type
            real_next_index += 1;
        }

        let mut insts = Vec::<Inst>::new();
        let mut has_a_return = false;
        for stmt in &func.block {
            match self.compile_stmt(stmt, &mut insts)? {
                Control::Return(_) => has_a_return = true,
                Control::ReturnScope => {
                    has_a_return = true;
                    eprintln!("{:?}", self.stack.top());
                }
                _ => {}
            }
        };

        // insert a return at the end if we don't have one
        if !has_a_return {
            tracing::warn!("auto-inserting return");
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

        // here we fix our return type types
        let mut ftype = func.ftype.clone();
        ftype.ret = match ftype.ret {
            Type::Scope(s) => {
                // all we're doing here is skipping over the arguments and captures and copying the data
                // verbatim otherwise.
                let mut scope_return_type = ospl_common::ast::Scope::default();
                for (key, type_location) in s.get_map() {
                    let ty = s.get_types().get(&type_location).expect("failed to get type? (this is an extreme bug)");
                    scope_return_type.declare(key.clone(), real_next_index, ty.clone());
                    real_next_index += 1;
                };

                Type::Scope(scope_return_type)
            },
            t => t,
        };

        let inst = InstBuilder::new()
            .opcode(Opc::PushFunction)
            .child(insts)
            .indexes(&capture_indexes)
            .build();

        ob.push(inst);

        self.stack.pop();

        return Ok(EvalResult {
            address: self.next_var(),
            ty: Type::Function(Box::new(ftype))
        })
    }

    pub fn do_call(
        &mut self,
        call_func: &Expression,
        args: &[Expression],
        ob: &mut Vec<Inst>,
    ) -> Res<EvalResult>
    {
        let f = self.eval(call_func, ob)?;
        let Type::Function(func) = f.ty
            else { unimplemented!() };  // FIXME: use result

        // TYPECHECKING...
        if func.args.len() != args.len() {
            // wrong number of args
            return Err(CE {
                at: Box::new(call_func.clone()),
                msg: None,
                error: CEData::WrongArgCount {
                    expected: func.args.len(),
                    got: args.len()
                }
            });
        }

        let mut new_args = Vec::new();
        for (arg, func_arg) in args.iter().zip(&func.args) {
            let eval = self.eval(arg, ob)?;
            if eval.ty != *func_arg {
                return Err(CE {
                    at: Box::new(arg.clone()),
                    msg: Some("perhaps you meant to cast the argument?"),
                    error: CEData::MismatchedTypes {
                        expected: crate::TypeExpectation::Exact(func_arg.clone()),
                        got: eval.ty
                    }
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