use ospl_common::{ast::{Expression, Scope, decl::Visibility, spanning::Spannable}, inst::optimized::{Inst, InstBuilder, Opc}};

use crate::{CE, CEData, Compiler, EvalResult, Res, Type, ast::FunctionValue, impls::stmt::Control};

impl Compiler {
    pub fn fn_literal(
        &mut self,
        func: &FunctionValue,
        span: &dyn Spannable,
        ob: &mut Vec<Inst>
    ) -> Res<EvalResult>
    {
        let mut new_scope = Scope::default();

        // add the args
        let mut arg_indexes = Vec::new();
        assert_eq!(func.ftype.args.len(), func.args.len());
        for (arg, ty) in func.args.iter().zip(func.ftype.args.iter()) {
            let variable = new_scope.next_post();

            // resolve
            let ty = self.rt(ty, span)?;

            new_scope.declare(arg.name.clone(), variable, ty);
            arg_indexes.push(variable);
        }

        // add the captures
        let mut capture_indexes = Vec::new();
        for capture in &func.captures {
            let variable = new_scope.next_post();

            let (u, t) = {
                // allow using the arguments here
                let scope = 
                    self.stack.scopes.last()
                        .ok_or_else(|| CE {
                            at: span.spanned(),
                            msg: None,
                            error: CEData::NoScopeToCapture,
                        })?;

                scope.get_combined_copy(&capture)
                    .ok_or_else(|| CE {
                        at: span.spanned(),
                        msg: Some("perhaps you meant to chain the capture across multiple scopes?"),
                        error: CEData::NotFoundInScope {
                            needed: capture.clone(),
                            scope: scope.clone()
                        }
                    })?
            };
            new_scope.declare(capture.clone(), variable, t);

            capture_indexes.push(u);
        }

        // PUSH HERE
        self.stack.scopes.push(new_scope);

        let mut insts = Vec::<Inst>::new();
        let mut has_a_return = false;
        for stmt in &func.block {
            match self.compile_stmt(stmt, &mut insts)? {
                Control::Return(_) => has_a_return = true,
                Control::ReturnScope => has_a_return = true,
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

        let mut new_scope = self.stack.scopes.pop()
            .expect("TODO unwrap - cannot return from the top of a script");

        // here we fix our return type types
        let mut ftype = func.ftype.clone();
        ftype.ret = match ftype.ret {
            Type::Scope(_) => {
                for s in &func.args {
                    if s.privacy != Visibility::Public {
                        new_scope.delete(&s.name);
                        unimplemented!("TODO impl - implement public/private args")
                    }
                }

                Type::Scope(new_scope)
            },
            t => t,
        };

        let inst = InstBuilder::new()
            .opcode(Opc::PushFunction)
            .child(insts)
            .indexes(&capture_indexes)
            .build();

        ob.push(inst);

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
            if !self.check_type(&eval.ty, func_arg, call_func)? {
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