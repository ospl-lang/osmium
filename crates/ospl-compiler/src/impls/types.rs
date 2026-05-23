use ospl_common::ast::{Type, spanning::Spannable};

use crate::{CE, CEData, Compiler, Res};

impl Compiler {
    fn check_type_of_var(&self, a1: &String, a2: &Type, _span: &dyn Spannable) -> Res<bool> {
        let (_, t) = self.stack.top().get_combined(a1).expect("TODO unwrap");
        return Ok(t == a2)
    }

    fn check_rt_of_fn(&self, a1: &Type, a2: &Type, span: &dyn Spannable) -> Res<bool> {
        match a1 {
            Type::Function(f) => {
                self.check_type(&f.ret, a2, span)
            }

            Type::TypeOfVar(name) => {
                let (_, t) = self.stack.top()
                    .get_combined(name)
                    .expect("TODO proper CE");

                self.check_rt_of_fn(&t, a2, span)
            }

            _ => todo!("emit proper CE"),
        }
    }

    pub fn check_type(&self, t1: &Type, t2: &Type, span: &dyn Spannable) -> Res<bool> {
        let t1 = &self.rt(t1, span)?;
        let t2 = &self.rt(t2, span)?;

        return Ok(match (t1, t2) {
            // special rule: Unknown matches everything
            (Type::Unknown, _) | (_, Type::Unknown) => true,

            // normal structural equality
            (Type::Nul, Type::Nul) => true,
            (Type::Undefined, Type::Undefined) => true,
            (Type::Int, Type::Int) => true,
            (Type::Address, Type::Address) => true,
            (Type::Float, Type::Float) => true,
            (Type::Str, Type::Str) => true,
            (Type::Char, Type::Char) => true,
            (Type::Bool, Type::Bool) => true,

            (Type::List(a), Type::List(b)) => a == b,
            (Type::Scope(a), Type::Scope(b)) => a == b,
            (Type::Function(a), Type::Function(b)) => a == b,

            (Type::ForeignLibrary, Type::ForeignLibrary) => true,

            (Type::ForeignFunction(args1, ret1), Type::ForeignFunction(args2, ret2)) => {
                args1 == args2 && ret1 == ret2
            }

            (Type::TypeOfVar(a1), a2) => self.check_type_of_var(a1, a2, span)?,
            (a2, Type::TypeOfVar(a1)) => self.check_type_of_var(a1, a2, span)?,

            (Type::ReturnTypeOf(a1), a2) => self.check_rt_of_fn(a1, a2, span)?,
            (a2, Type::ReturnTypeOf(a1)) => self.check_rt_of_fn(a1, a2, span)?,

            _ => false,
        });
    }

    pub fn rt(&self, ty: &Type, span: &dyn Spannable) -> Res<Type> {
        match ty {
            Type::TypeOfVar(name) => {
                let Some((_, t)) = self.stack.top().get_combined(name)
                else { return Err(CE {
                    at: span.spanned(),
                    error: CEData::NotFoundInScope { needed: name.clone(), scope: self.stack.top().clone() },
                    msg: None,
                }) };

                self.rt(&t, span)
            }

            Type::ReturnTypeOf(inner) => {
                let resolved = self.rt(inner, span)?;

                match resolved {
                    Type::Function(f) => {
                        self.rt(&f.ret, span)
                    }

                    _ => todo!("TODO error"),
                }
            }

            _ => Ok(ty.clone()),
        }
    }
}
