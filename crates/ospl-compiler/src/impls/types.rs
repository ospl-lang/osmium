use ospl_common::ast::{FunctionType, Scope, Type, UType, spanning::Spannable};

use crate::{CE, CEData, Compiler, Res};

impl Compiler {
    pub fn check_type(&self, t1: &Type, t2: &Type) -> bool {
        return match (t1, t2) {
            // special rule: Unknown matches everything
            (Type::Unknown, _) | (_, Type::Unknown) => true,

            // special rule: Undefined only matches itself
            // but is legal to compare to all other objects
            (Type::Undefined, Type::Undefined) => true,
            (Type::Undefined, _) => false,
            (_, Type::Undefined) => false,

            // normal structural equality
            (Type::Nul, Type::Nul) => true,
            (Type::Int, Type::Int) => true,
            (Type::Address, Type::Address) => true,
            (Type::Float, Type::Float) => true,
            (Type::Str, Type::Str) => true,
            (Type::Char, Type::Char) => true,
            (Type::Bool, Type::Bool) => true,

            (Type::List(a), Type::List(b)) => a == b,
            (Type::Scope(a), Type::Scope(b)) => a >= b,
            (Type::Function(a), Type::Function(b)) => a == b,

            (Type::ForeignLibrary, Type::ForeignLibrary) => true,

            (Type::ForeignFunction(args1, ret1), Type::ForeignFunction(args2, ret2)) => {
                args1 == args2 && ret1 == ret2
            }

            _ => false,
        };
    }

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
