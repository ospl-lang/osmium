use ospl_common::{ast::spanning::Spannable, inst::{make, optimized::{Inst, InstBuilder, Opc}}};
use tracing::error;

use crate::{CE, CEData, Compiler, EvalResult, Res, Type, TypeExpectation, ast::{Expr, LV, LValue, Literal}, impls::stmt::Control};

impl Compiler {
    pub fn eval(
        &mut self,
        expr: &crate::ast::Expression,
        ob: &mut Vec<Inst>
    ) -> Res<EvalResult>
    {
        match &*expr.inner {
            Expr::Literal(l) => self.literal(l, expr, ob),
            Expr::Call { left, args, named_args } => self.do_fn_call(left, args, named_args, ob),
            Expr::BinaryOp(b) => self.binary_op(b, ob),
            Expr::UnaryOp(u) => self.unary_op(u, ob, expr),
            Expr::LValue(lv) => {
                let store = self.get_lvalue(lv, ob)?;
                return Ok(EvalResult {
                    address: store.address,
                    ty: store.ty.clone()
                })
            },
            Expr::FFILoad(f) => {
                let fp = self.eval(f, ob)?;
                if fp.ty != Type::Str {
                    /* error */
                }

                let i = InstBuilder::new()
                    .opcode(Opc::FFILoadLib)
                    .index(fp.address)
                    .symbol(&mut *self.bd.symbols.lock()?, expr.user_symbol())
                    .build();

                ob.push(i);

                return Ok(EvalResult {
                    address: self.next_var(),
                    ty: Type::ForeignLibrary
                })
            },
            Expr::FFICall(f, args) => self.ffi_call(f, args, ob),
            Expr::FFIFunc(lib, func_name, rtype, types) => self.ffi_func(lib, func_name, *rtype, types, ob),
            Expr::Apply(left, nominals_to_replacements) => {
                let eval = self.eval(left, ob)?;
                let ty = self.nominal_application(eval.ty, nominals_to_replacements, expr)?;
                return Ok(EvalResult {
                    address: eval.address,
                    ty: ty
                })
            }
            Expr::Cast(left, into) => {
                let left = self.eval(left, ob)?;
                let new_into = self.rt(self.stack.top(), into, expr)?;

                // for turning INTO a nominal
                if let Type::Nominal(_, p) = &new_into {
                    if left.ty != **p {
                        return Err(CE {
                            at: expr.spanned(),
                            error: CEData::MismatchedTypes {
                                expected: TypeExpectation::Exact(*p.clone()),
                                got: left.ty
                            },
                            during: "Nominal cast - type check",
                            msg: None
                        })
                    }

                    return Ok(EvalResult {
                        address: left.address,
                        ty: new_into.clone()
                    })
                }

                // for UNWRAPPING a nominal
                else if let Type::Nominal(_, p) = &left.ty {
                    if new_into != **p {
                        return Err(CE {
                            at: expr.spanned(),
                            error: CEData::MismatchedTypes {
                                expected: TypeExpectation::Exact(new_into.clone()),
                                got: *p.clone()
                            },
                            during: "Nominal unwrap cast - type check",
                            msg: None
                        })
                    }

                    return Ok(EvalResult {
                        address: left.address,
                        ty: new_into.clone(),
                    })
                }

                // for unwrapping a union
                else if let Type::Union(_, _) = &left.ty {
                    if new_into != left.ty {
                        return Err(CE {
                            at: expr.spanned(),
                            during: "Union unwrap cast - type check",
                            error: CEData::UnionDoesntHaveType { union: left.ty, doesnt_have: new_into },
                            msg: None
                        });
                    }

                    return Ok(EvalResult {
                        address: left.address,
                        ty: new_into
                    })
                }
                
                else {
                    ob.push(InstBuilder::new()
                        .opcode(Opc::Cast)
                        .index(left.address)
                        .index(new_into.to_primitive_type_id())
                        .symbol(&mut *self.bd.symbols.lock()?, expr.user_symbol())
                        .build());

                    return Ok(EvalResult {
                        address: self.next_var(),
                        ty: new_into.clone()
                    })
                }
            },
            Expr::Block(b) => {
                // This is a dummy evalresult that we know will be overwritten.
                let mut eval = None;
                for s in b {
                    match self.compile_stmt(s, ob)? {
                        Control::Return(y) => {
                            eval = Some(y);
                        },
                        Control::ReturnScope => {
                            // error: unsupported
                            todo!("error unsupported")
                        }
                        _ => {}
                    }
                }

                if eval.is_none() {
                    ob.push(InstBuilder::new()
                        .opcode(Opc::PushLiteral)
                        .value(make::nul(()))
                        .build());

                    return Ok(EvalResult {
                        address: self.next_var(),
                        ty: Type::Nul
                    })
                }

                return Ok(eval.expect("bug"));
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
                ob.push(InstBuilder::new().opcode(Opc::PushLiteral).value(make::int(*i)).build());
                return Ok(EvalResult { address: self.next_var(), ty: Type::Int })
            },
            Literal::Address(u) => {
                ob.push(InstBuilder::new().opcode(Opc::PushLiteral).value(make::addr(*u)).build());
                return Ok(EvalResult { address: self.next_var(), ty: Type::Address })
            },
            Literal::Float(f) => {
                ob.push(InstBuilder::new().opcode(Opc::PushLiteral).value(make::float(*f)).build());
                return Ok(EvalResult { address: self.next_var(), ty: Type::Float })
            },
            Literal::Bool(b) => {
                ob.push(InstBuilder::new().opcode(Opc::PushLiteral).value(make::bool(*b)).build());
                return Ok(EvalResult { address: self.next_var(), ty: Type::Bool })
            },
            Literal::Str(s) => {
                ob.push(InstBuilder::new().opcode(Opc::PushLiteral).value(make::str(s.clone())).build());
                return Ok(EvalResult { address: self.next_var(), ty: Type::Str })
            },
            Literal::Char(c) => {
                ob.push(InstBuilder::new().opcode(Opc::PushLiteral).value(make::char(*c)).build());
                return Ok(EvalResult { address: self.next_var(), ty: Type::Char })
            },
            Literal::Nul => {
                ob.push(InstBuilder::new().opcode(Opc::PushLiteral).value(make::nul(())).build());
                return Ok(EvalResult { address: self.next_var(), ty: Type::Nul })
            },
            Literal::Undefined => {
                ob.push(InstBuilder::new().opcode(Opc::PushLiteral).value(make::undefined(())).build());
                return Ok(EvalResult { address: self.next_var(), ty: Type::Undefined })
            },
            Literal::List(lty, l) => {
                let lty = self.rt(self.stack.top(), lty, span)?;
                let mut indexes = Vec::new();
                for expr in l.iter() {
                    let eval = self.eval(expr, ob)?;
                    if eval.ty != lty {
                        /* error */
                        error!("a list literal's types must match the declared type, got {:?} expected {:?}", eval.ty, lty);
                    }
                    indexes.push(eval.address);
                }

                ob.push(InstBuilder::new()
                    .opcode(Opc::PushArray)
                    .indexes(&indexes)
                    .build());

                return Ok(EvalResult { address: self.next_var(), ty: Type::List(Box::new(lty)) })
            },
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
                let x = match eval.ty {
                    Type::Scope(s) => {
                        let v = s.get_runtime(var)
                            // not found in that scope
                            .ok_or_else(|| CE {
                                at: Box::new(lv2.clone()),
                                during: "LValue retrival - scope access",
                                msg: Some("perhaps you typed the wrong name?"),
                                error: CEData::NotFoundInScope {
                                    needed: var.to_string(),
                                    scope: s.clone(),
                                }
                            })?;

                        let x = EvalResult {
                            address: v.get_address(),
                            ty: v.get_type().clone()
                        };

                        x
                    },
                    t @ (Type::List(_) | Type::Str) => {
                        match var.as_str() {
                            "len" => {
                                ob.push(InstBuilder::new()
                                    .opcode(Opc::GetLength)
                                    .index(eval.address)
                                    .build());

                                return Ok(EvalResult { address: self.next_var(), ty: Type::Address })
                            },

                            other => return Err(CE {
                                at: Box::new(lv2.clone()),
                                during: "LValue retrival - special var access",
                                error: CEData::UnrecognizedSpecialVar {
                                    ty: t.clone(),
                                    special: other.to_string()
                                },
                                msg: Some("valid special props here: len"),
                            })
                        };
                    }
                    other => return Err(CE {
                        at: Box::new(lv2.clone()),
                        msg: Some("perhaps you're accessing the wrong value?"),
                        during: "LValue retrival - property access (invalid)",
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
                    .symbol(&mut *self.bd.symbols.lock()?, lv.user_symbol())
                    .build();

                ob.push(i);

                return Ok(EvalResult {
                    // yes this is correct just believe me.
                    address: self.next_var(),
                    ty: x.ty
                })
            },

            LV::Index(l, r) => self.c_index(l, r, lv, ob),
            LV::Slice(l, r1, r2) => self.c_slice(l, r1, r2, lv, ob),

            // FIXME unwrap
            LV::Variable(var) => {
                let v = self.stack.top().get_runtime(var)
                    .ok_or_else(|| CE {
                        at: Box::new(lv.clone()),
                        msg: Some("Did you type the wrong variable name"),
                        during: "LValue retrival - variable access",
                        error: CEData::NotFoundInScope {
                            needed: var.to_string(),
                            scope: self.stack.top().clone()
                        }
                    })?;

                let x = EvalResult {
                    address: v.get_address(),
                    ty: v.get_type().clone()
                };
                return Ok(x)
            },
        }
    }
}
