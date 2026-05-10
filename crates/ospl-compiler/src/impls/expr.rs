use ospl_common::{ast::{FunctionType, spanning::Spannable}, inst::optimized::{Inst, InstBuilder, Opc}};
use tracing::error;

use crate::{CE, CEData, Compiler, EvalResult, Res, Type, ast::{Expr, LV, LValue, Literal}};

impl Compiler {
    pub fn eval(
        &mut self,
        expr: &crate::ast::Expression,
        ob: &mut Vec<Inst>
    ) -> Res<EvalResult>
    {
        match &*expr.inner {
            Expr::Literal(l) => self.literal(l, expr, ob),
            Expr::Call(func, args) => self.do_call(func, args, ob),
            Expr::BinaryOp(b) => self.binary_op(b, ob),
            Expr::UnaryOp(u) => self.unary_op(u, ob),
            Expr::LValue(lv) => {
                let store = self.get_lvalue(lv, ob)?;
                return Ok(EvalResult {
                    address: store.address,
                    ty: store.ty.clone()
                })
            },
            Expr::FFICall(f, args) => self.ffi_call(f, args, ob),
            Expr::FFIFunc(lib, func_name, rtype, types) => self.ffi_func(lib, func_name, *rtype, types, ob),
            Expr::FFILoad(lib_path) => self.ffi_load(lib_path, ob),
            Expr::Cast(left, into) => {
                let left = self.eval(left, ob)?;

                ob.push(InstBuilder::new()
                    .opcode(Opc::Cast)
                    .index(left.address)
                    .index(into.to_primitive_type_id())
                    .build());

                return Ok(EvalResult {
                    address: self.next_var(),
                    ty: into.clone()
                })
            }
            Expr::Use(pkg) => {
                // SAFETY: I promise not to mutate p.cached or p.code inside
                // any calls to `&mut self` methods on [`Compiler`]
                //
                // in other terms, I promise this code is synchronous with
                // respect to `p`.
                let p = &raw const *self.get_module(pkg).expect("TODO unwrap");
                let p = unsafe{&*p};

                if let Some(cached) = &p.cached {
                    return Ok(cached.clone())
                }

                // otherwise we have to construct it
                self.stack.push();
                let mut code = Vec::new();
                self.compile_block(&p.code, &mut code)?;

                // create an IIFE
                code.push(InstBuilder::new()
                    .opcode(Opc::RetScope)
                    .build());

                let mut iife = Vec::new();
                iife.push(InstBuilder::new()
                    .opcode(Opc::PushFunction)
                    .child(code)
                    .build());

                let ret = Type::Scope(self.stack.top().clone());
                self.stack.pop();
                // not declared on compiler's end
                let func_eval = EvalResult {
                    address: self.next_var(),
                    ty: Type::Function(Box::new(FunctionType {
                        generics: Vec::new(),
                        args: Vec::new(),
                        ret,
                    })),
                };

                iife.push(InstBuilder::new()
                    .opcode(Opc::Call)
                    .index(func_eval.address)
                    .build());

                let call_eval = EvalResult {
                    address: self.next_var(),
                    ty: if let Type::Function(ftyp) = func_eval.ty {
                        ftyp.ret
                    } else { unreachable!("what the helly?") }
                };

                // quickly! Ccahe it!
                let p = self.get_mut_module(pkg).expect("TODO unwrap");
                p.cached = Some(call_eval.clone());

                ob.append(&mut iife);

                return Ok(call_eval)
            }
        }
    }

    pub fn literal(
        &mut self,
        l: &Literal,
        span: &dyn Spannable,
        ob: &mut Vec<Inst>
    ) -> Res<EvalResult>
    {
        match l {
            Literal::Function(f) => self.fn_literal(f, span, ob),

            // no idea how to write this without duplicating code.. if anyone knows a cleaner way LMK
            Literal::Int(i) => {
                ob.push(InstBuilder::new().opcode(Opc::PushLiteral).value(ospl_common::inst::RuntimeValue::Int(*i)).build());
                return Ok(EvalResult { address: self.next_var(), ty: Type::Int })
            },
            Literal::Address(u) => {
                ob.push(InstBuilder::new().opcode(Opc::PushLiteral).value(ospl_common::inst::RuntimeValue::Address(*u)).build());
                return Ok(EvalResult { address: self.next_var(), ty: Type::Address })
            },
            Literal::Float(f) => {
                ob.push(InstBuilder::new().opcode(Opc::PushLiteral).value(ospl_common::inst::RuntimeValue::Float(*f)).build());
                return Ok(EvalResult { address: self.next_var(), ty: Type::Float })
            },
            Literal::Bool(b) => {
                ob.push(InstBuilder::new().opcode(Opc::PushLiteral).value(ospl_common::inst::RuntimeValue::Bool(*b)).build());
                return Ok(EvalResult { address: self.next_var(), ty: Type::Bool })
            },
            Literal::Str(s) => {
                ob.push(InstBuilder::new().opcode(Opc::PushLiteral).value(ospl_common::inst::RuntimeValue::Str(s.clone())).build());
                return Ok(EvalResult { address: self.next_var(), ty: Type::Str })
            },
            Literal::Nul => {
                ob.push(InstBuilder::new().opcode(Opc::PushLiteral).value(ospl_common::inst::RuntimeValue::Nul).build());
                return Ok(EvalResult { address: self.next_var(), ty: Type::Nul })
            },
            Literal::Undefined => {
                ob.push(InstBuilder::new().opcode(Opc::PushLiteral).value(ospl_common::inst::RuntimeValue::Undefined).build());
                return Ok(EvalResult { address: self.next_var(), ty: Type::Undefined })
            },
            Literal::List(lty, l) => {
                let mut indexes = Vec::new();
                for expr in l.iter() {
                    let eval = self.eval(expr, ob)?;
                    if !self.check_type(&eval.ty, lty, span)? {
                        /* error */
                        error!("a list literal's types must match the declared type, got {:?} expected {:?}", eval.ty, lty);
                    }
                    indexes.push(eval.address);
                }

                ob.push(InstBuilder::new()
                    .opcode(Opc::PushArray)
                    .indexes(&indexes)
                    .build());

                return Ok(EvalResult { address: self.next_var(), ty: Type::List(Box::new(lty.clone())) })
            }
        }
    }

    pub fn get_lvalue(
        &mut self,
        lv: &LValue,
        ob: &mut Vec<Inst>
    ) -> Res<EvalResult>
    {
        match &*lv.inner {
            LV::Property(lv2, var) => {
                let eval = self.get_lvalue(lv2, ob)?;
                let ty = self.rt(&eval.ty, lv2)?;
                let x = match ty {
                    Type::Scope(s) => {
                        let v = s.get_combined(var)
                            // not found in that scope
                            .ok_or_else(|| CE {
                                at: Box::new(lv2.clone()),
                                msg: Some("perhaps you typed the wrong name?"),
                                error: CEData::NotFoundInScope {
                                    needed: var.to_string(),
                                    scope: s.clone(),
                                }
                            })?;

                        let x = EvalResult::from(v);
                        x
                    },
                    list_ty @ Type::List(_) => {
                        match var.as_str() {
                            "len" => {
                                ob.push(InstBuilder::new()
                                    .opcode(Opc::GetLength)
                                    .index(eval.address)
                                    .build());

                                return Ok(EvalResult { address: self.next_var(), ty: Type::Int })
                            },

                            other => return Err(CE {
                                at: Box::new(lv2.clone()),
                                error: CEData::UnrecognizedSpecialVar {
                                    ty: list_ty.clone(),
                                    special: other.to_string()
                                },
                                msg: Some("valid special props here: len"),
                            })
                        };
                    }
                    other => return Err(CE {
                        at: Box::new(lv2.clone()),
                        msg: Some("perhaps you're accessing the wrong value?"),
                        error: CEData::MismatchedTypes {
                            expected: crate::TypeExpectation::AnyScope,
                            got: other.clone()
                        }
                    })
                };

                let i = InstBuilder::new()
                    .opcode(Opc::Property)
                    .index(eval.address)
                    .index(x.address)
                    .build();

                ob.push(i);

                return Ok(EvalResult {
                    // yes this is correct just believe me.
                    address: self.next_var(),
                    ty: x.ty
                })
            },

            LV::Index(l, r) => self.array_index(l, r, ob),
            LV::Slice(l, r1, r2) => self.array_slice(l, r1, r2, ob),

            // FIXME unwrap
            LV::Variable(var) => {
                let v = self.stack.top().get_combined(var)
                    .ok_or_else(|| CE {
                        at: Box::new(lv.clone()),
                        msg: Some("Perhaps you failed preschool?"),
                        error: CEData::NotFoundInScope {
                            needed: var.to_string(),
                            scope: self.stack.top().clone()
                        }
                    })?;

                let x = EvalResult::from(v);
                return Ok(x)
            },
        }
    }

    // pub fn literal_generic(
    //     &mut self,
    //     l: RuntimeValue,
    //     ob: &mut Vec<Inst>
    // ) -> EvalResult
    // {
    //     let i = InstBuilder::new()
    //         .opcode(Opc::PushLiteral)
    //         .value(l)
    //         .build();

    //     ob.push(i);
    //     return EvalResult {
    //         address: self.next_var(),
    //         ty: Type::from_runtime(l)
    //     }
    // }
}
