use std::collections::BTreeMap;

use ospl_common::ast::{FunctionType, Scope, Type, UType, spanning::Spannable};

use crate::{CE, CEData, Compiler, Res};

impl Compiler {
    pub fn rt(&self, scope: &Scope<Type>, ty: &UType, span: &dyn Spannable) -> Res<Type> {
        match ty {
            UType::Typeof(name) => {
                let Some(t) = scope.get_entry_type(name)
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

                    _ => return Err(CE {
                        at: span.spanned(),
                        during: "Type resolution - Return type of",
                        error: CEData::TypeDoesntHaveAReturn { t: resolved },
                        msg: None
                    })
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
                        let Some(bv) = s.get_entry_type(p)
                        else { return Err(CE {
                            at: span.spanned(),
                            during: "Type resolution - Property",
                            error: CEData::NotFoundInScope { needed: p.clone(), scope: s.clone() },
                            msg: None,
                        }) };

                        tracing::debug!("getting type property: {bv:?}");

                        return Ok(bv.clone())
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

                let mut named_args = BTreeMap::new();
                for (name, arg) in &f.named_args {
                    named_args.insert(
                        name.clone(),
                        self.rt(scope, &arg, span)?
                    );
                }

                return Ok(Type::Function(Box::new(FunctionType {
                    ret,
                    named_args,
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
                    let ent = value.clone().map_type_into_resultant(|e| {
                        self.rt(scope, &e, span)
                    })?;
                    s2.direct_declare(key.clone(), ent);
                };
                return Ok(Type::Scope(s2))
            },

            UType::InferScope => {
                return Ok(Type::Scope(scope.clone()))
            },

            UType::Union(u, t) => {
                let utt = self.rt(scope, &*u, span)?;
                let rtt = self.rt(scope,&*t, span)?;
                return Ok(Type::Union(Box::new(utt), Box::new(rtt)));
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

                let mut new_named_args = BTreeMap::new();
                for (name, arg) in f.named_args {
                    new_named_args.insert(name, self.nominal_application(arg, replacements, span)?);
                }

                return Ok(Type::Function(Box::new(FunctionType {
                    args: new_args,
                    named_args: new_named_args,
                    ret: self.nominal_application(f.ret, replacements, span)?
                })))
            },
            Type::List(l) => {
                return Ok(Type::List(Box::new(self.nominal_application(*l, replacements, span)?)))
            },
            Type::Scope(s) => {
                let mut s2 = Scope::default();
                for (name, ent) in s.into_inner() {
                    s2.direct_declare(name, ent.map_type_into_resultant(|ent| {
                        self.nominal_application(ent, replacements, span)
                    })?);
                }

                return Ok(Type::Scope(s2))
            }
            other => Ok(other),
        }
    }
}
