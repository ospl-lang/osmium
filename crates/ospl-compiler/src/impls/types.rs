use ospl_common::ast::{FunctionType, Scope, Type, UType, spanning::Spannable};

use crate::{CE, CEData, Compiler, Res};

impl Compiler {
    pub fn rt(&self, scope: &Scope<Type>, ty: &UType, span: &dyn Spannable) -> Res<Type> {
        match ty {
            UType::Typeof(name) => {
                let Some((_, t)) = scope.get_combined_with_nonaddressable(name)
                else { return Err(CE {
                    at: span.spanned(),
                    during: "Type resolution - TypeOfVar",
                    error: CEData::NotFoundInScope { needed: name.clone(), scope: scope.clone() },
                    msg: None,
                }) };

                return Ok(t.clone())
            }

            UType::Returnof(inner) => {
                let resolved = self.rt(scope, inner, span)?;

                match resolved {
                    Type::Function(f) => {
                        return Ok(f.ret.clone())
                    },

                    Type::Nominal(_, curr) => {
                        return Ok(*curr.clone())
                    },

                    _ => todo!("TODO error"),
                }
            },

            UType::Nominal(n) => {
                let typ = self.rt(scope, &**n, span)?;
                let nom = self.bd.next_resource_id.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
                return Ok(Type::Nominal(nom, Box::new(typ)));
            },

            UType::Apply(nom, map) => {
                let ty = self.rt(scope, nom, span)?;
                let applied = self.nominal_application(ty, map, span)?;
                return Ok(applied)
            }

            UType::Property(b, p) => {
                let br = self.rt(scope, &b, span)?;
                match br {
                    Type::Scope(s) => {
                        let Some(bv) = s.get_inner().get(p)
                        else { return Err(CE {
                            at: span.spanned(),
                            during: "Type resolution - Property",
                            error: CEData::NotFoundInScope { needed: p.clone(), scope: s.clone() },
                            msg: None,
                        }) };

                        tracing::debug!("getting type property: {bv:?}");

                        return Ok(bv.get_type().clone())
                    },
                    _ => return Err(CE {
                        at: span.spanned(),
                        during: "Type resolution - Property",
                        error: CEData::MismatchedTypes { expected: crate::TypeExpectation::AnyScope, got: br.clone() },
                        msg: None,
                    }) 
                }
            }

            UType::Resolved(r) => return Ok(r.clone()),

            UType::Function(f) => {
                let ret = self.rt(scope, &f.ret, span)?;

                let mut args = Vec::new();
                for arg in &f.args {
                    args.push(self.rt(scope, arg, span)?);
                }

                return Ok(Type::Function(Box::new(FunctionType {
                    ret,
                    args
                })))
            },

            UType::List(lty) => {
                let lty = self.rt(scope, lty, span)?;
                return Ok(Type::List(Box::new(lty)))
            },

            UType::Scope(s) => {
                let mut s2: Scope<Type> = Scope::default();
                for (key, value) in s.get_inner() {
                    let t = value.get_type();
                    let t = self.rt(scope, t, span)?;
                    s2.direct_declare(key.clone(), value.get_address(), t);
                };
                return Ok(Type::Scope(s2))
            },

            UType::InferScope => {
                return Ok(Type::Scope(scope.clone()))
            }
        }
    }

    pub fn nominal_application(&self, ty: Type, replacements: &Vec<(UType, UType)>, span: &dyn Spannable) -> Res<Type> {
        match ty {
            Type::Nominal(nom_nom, _) => {
                for (find, replace) in replacements {
                    let find = self.rt(self.stack.top(), find, span)?;
                    if let Type::Nominal(nom, _) = find {
                        if nom_nom == nom {
                            let ty = self.rt(self.stack.top(), replace, span)?;
                            return Ok(ty)
                        }
                    }
                }

                return Ok(ty);
            }
            Type::Function(f) => {
                let mut new_args = Vec::new();
                for arg in f.args {
                    new_args.push(self.nominal_application(arg, replacements, span)?);
                }

                return Ok(Type::Function(Box::new(FunctionType {
                    args: new_args,
                    ret: self.nominal_application(f.ret, replacements, span)?
                })))
            },
            Type::List(l) => {
                return Ok(Type::List(Box::new(self.nominal_application(*l, replacements, span)?)))
            },
            Type::Scope(s) => {
                let mut s2 = Scope::default();
                for (name, stor) in s.into_inner() {
                    let address = stor.get_address();
                    let ty = self.nominal_application(stor.into_type(), replacements, span)?;
                    s2.direct_declare(name, address, ty);
                }

                return Ok(Type::Scope(s2))
            }
            other => Ok(other),
        }
    }
}
