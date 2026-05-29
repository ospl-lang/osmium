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
                    }

                    _ => todo!("TODO error"),
                }
            },

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
}
