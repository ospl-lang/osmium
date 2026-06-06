use ospl_common::{ast::{Expression, FunctionType, Scope, spanning::Spannable}, inst::{make, optimized::{Inst, InstBuilder, Opc}}};

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

        // add the captures
        let mut capture_indexes = Vec::new();
        for capture in &func.captures {
            let (u, t) = {
                let scope = 
                    self.stack.scopes.last()
                        .ok_or_else(|| CE {
                            at: span.spanned(),
                            msg: None,
                            during: "capture handling",
                            error: CEData::NoScopeToCapture,
                        })?;

                let x = scope.get_combined_with_nonaddressable(&capture);
                match x {
                    // capture nonaddressables
                    Some((None, ty)) => {
                        new_scope.declare_non_addressable(capture.clone(), ty.clone());
                        continue;
                    },

                    Some((Some(x), ty)) => (x, ty.clone()),

                    _ => return Err(CE {
                        at: span.spanned(),
                        msg: Some("perhaps you meant to chain the capture across multiple scopes?"),
                        during: "capture handling",
                        error: CEData::NotFoundInScope {
                            needed: capture.clone(),
                            scope: scope.clone()
                        }
                    }),
                }
            };

            let variable = new_scope.next_post();
            new_scope.declare(capture.clone(), variable, t);
            capture_indexes.push(u);
        }

        // add the args
        let mut arg_types = Vec::new();
        assert_eq!(func.ftype.args.len(), func.args.len());
        for (arg, ty) in func.args.iter().zip(func.ftype.args.iter()) {
            let variable = new_scope.next_post();

            let t = self.rt(&new_scope, ty, span)?;

            new_scope.declare(arg.name.clone(), variable, t.clone());
            arg_types.push(t);
        }

        // PUSH HERE
        self.stack.scopes.push(new_scope);

        let mut insts = Vec::<Inst>::new();
        let mut has_a_return = false;
        for stmt in &func.block {
            match self.compile_stmt(stmt, &mut insts)? {
                Control::Return(_) => has_a_return = true,
                Control::ReturnScope => {
                    has_a_return = true;
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
                .value(make::nul(()))
                .build());

            insts.push(InstBuilder::new()
                .opcode(Opc::Ret)
                .index(v)
                .build());
        }

        let new_scope = self.stack.scopes.pop()
            .expect("TODO unwrap - cannot return from the top of a script");

        // here we fix our function type
        let ret = self.rt(&new_scope, &func.ftype.ret, span)?;

        tracing::trace!("before fixing: {ret:?}");

        let new_type = FunctionType {
            args: arg_types,
            ret
        };

        tracing::trace!("function {new_type}");

        let inst = InstBuilder::new()
            .opcode(Opc::PushFunction)
            .child(insts)
            .indexes(&capture_indexes)
            .symbol(&mut *self.bd.symbols.lock()?, span.user_symbol())
            .build();

        ob.push(inst);

        return Ok(EvalResult {
            address: self.next_var(),
            ty: Type::Function(Box::new(new_type))
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
        let func = match f.ty {
            Type::Function(f) => *f,
            other => panic!("cannot call {other:?}")
        };

        // TYPECHECKING...
        if func.args.len() != args.len() {
            // wrong number of args
            return Err(CE {
                at: Box::new(call_func.clone()),
                msg: None,
                during: "function call instance",
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
                    during: "function call instance",
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
            .symbol(&mut *self.bd.symbols.lock()?, call_func.user_symbol())
            .build();

        ob.push(i);

        return Ok(EvalResult {
            address: self.next_var(),
            ty: func.ret.clone()
        })
    }
}