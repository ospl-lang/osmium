use std::collections::BTreeMap;

use ospl_common::{ast::{Entry, Expression, FunctionType, RStore, Scope, spanning::Spannable}, inst::{make, optimized::{Inst, InstBuilder, Opc}}};

use crate::{CE, CEData, Compiler, EvalResult, Res, Type, ast::FunctionValue, impls::stmt::Control};

impl Compiler {
    /// Compiles the given function literal into `ob`,
    /// using `span` for errors
    /// 
    ///  ## Calling convention
    /// The calling convention here is the same as the normal OSPL calling
    /// convention, but the named args of functions go before the args in
    /// the same index list.
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
            let scope = 
                self.stack.scopes.last()
                    .ok_or_else(|| CE {
                        at: span.spanned(),
                        msg: None,
                        during: "capture handling",
                        error: CEData::NoScopeToCapture,
                    })?;

            let x = scope.get_entry(&capture);
            match x {
                // capture nonaddressables
                Some(a @ Entry::Alias(_)) => {
                    new_scope.direct_declare(capture.clone(), a.clone());
                    continue;
                },

                Some(Entry::Runtime(rr)) => {
                    let variable = new_scope.next_post();
                    new_scope.direct_declare(capture.clone(), Entry::Runtime(RStore {
                        address: variable,
                        typ: rr.typ.clone()
                    }));
                    capture_indexes.push(rr.address);
                    continue;
                }

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
        }

        // add the named args
        let mut named_arg_types = BTreeMap::new();
        for (arg, ty) in func.ftype.named_args.iter() {
            let variable = new_scope.next_post();

            let rty = self.rt(&new_scope, ty, span)?;
            named_arg_types.insert(arg.clone(), rty.clone());
            
            new_scope.direct_declare(arg.clone(), Entry::Runtime(RStore {
                address: variable,
                typ: rty
            }));
        }

        // add the args
        let mut arg_types = Vec::new();
        assert_eq!(func.ftype.args.len(), func.args.len());
        for (arg, ty) in func.args.iter().zip(func.ftype.args.iter()) {
            let variable = new_scope.next_post();

            let t = self.rt(&new_scope, ty, span)?;

            new_scope.direct_declare(arg.name.clone(), Entry::Runtime(RStore {
                address: variable,
                typ: t.clone(),
            }));
            arg_types.push(t);
        }

        // PUSH HERE
        self.stack.scopes.push(new_scope);

        let mut insts = Vec::<Inst>::new();
        let mut has_a_return = false;
        for stmt in &func.block {
            match self.compile_stmt(stmt, &mut insts)? {
                Control::Return(_) => {
                    has_a_return = true;
                },
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

        let ret = self.rt(&new_scope, &func.ftype.ret, span)?;

        let new_type = FunctionType {
            args: arg_types,
            named_args: named_arg_types,
            ret
        };

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

    /// Preforms a call to a function `call_func` with `args` and `named_args`
    /// 
    /// See [`Compiler::fn_literal`] in the section "calling convention"
    pub fn do_fn_call(
        &mut self,
        call_func: &Expression,
        args: &[Expression],
        named_args: &BTreeMap<String, Expression>,
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

        // named args
        let mut new_named_args = BTreeMap::new();
        for (argname, argexpr) in named_args.iter() {
            let e = self.eval(argexpr, ob)?;
            new_named_args.insert(argname.clone(), e);
        }

        // TYPECHECKING
        let named_args_types: BTreeMap<_, _> = new_named_args
            .iter()
            .map(|(a, b)| (a.clone(), b.ty.clone()))
            .collect();

        if named_args_types != func.named_args {
            // wrong number of args
            return Err(CE {
                at: Box::new(call_func.clone()),
                msg: None,
                during: "function call instance",
                error: CEData::WrongNamedArgs {
                    expected: func.named_args.clone(),
                    got: named_args_types,
                }
            });
        }

        // named args
        let new_named_args_indexes: Vec<usize> = new_named_args
            .iter()
            .map(|(_, b)| b.address)
            .collect();

        // args
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
            .indexes(&new_named_args_indexes)
            .symbol(&mut *self.bd.symbols.lock()?, call_func.user_symbol())
            .build();

        ob.push(i);

        return Ok(EvalResult {
            address: self.next_var(),
            ty: func.ret.clone()
        })
    }
}