use ospl_common::ast::{FunctionType, Scope, Type, spanning::Spannable};

use crate::{CE, CEData, Compiler, Res};

impl Compiler {
    fn check_type_of_var(&self, scope: &Scope, a1: &String, a2: &Type, _span: &dyn Spannable) -> Res<bool> {
        let (_, t) = scope.get_combined_with_nonaddressable(a1).expect("TODO unwrap");
        return Ok(t == a2)
    }

    fn check_rt_of_fn(&self, scope: &Scope, a1: &Type, a2: &Type, span: &dyn Spannable) -> Res<bool> {
        match a1 {
            Type::Function(f) => {
                self.check_type(scope, &f.ret, a2, span)
            }

            Type::TypeOfVar(name) => {
                let (_, t) = scope
                    .get_combined_with_nonaddressable(name)
                    .expect("TODO proper CE");

                self.check_rt_of_fn(scope, &t, a2, span)
            }

            _ => todo!("emit proper CE"),
        }
    }

    pub fn check_type(&self, scope: &Scope, t1: &Type, t2: &Type, span: &dyn Spannable) -> Res<bool> {
        let t1 = &self.rt(scope, t1, span)?;
        let t2 = &self.rt(scope, t2, span)?;

        return Ok(match (t1, t2) {
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
            (Type::Scope(a), Type::Scope(b)) => a == b,
            (Type::Function(a), Type::Function(b)) => a == b,

            (Type::ForeignLibrary, Type::ForeignLibrary) => true,

            (Type::ForeignFunction(args1, ret1), Type::ForeignFunction(args2, ret2)) => {
                args1 == args2 && ret1 == ret2
            }

            (Type::TypeOfVar(a1), a2) => self.check_type_of_var(scope, a1, a2, span)?,
            (a2, Type::TypeOfVar(a1)) => self.check_type_of_var(scope, a1, a2, span)?,

            (Type::ReturnTypeOf(a1), a2) => self.check_rt_of_fn(scope, a1, a2, span)?,
            (a2, Type::ReturnTypeOf(a1)) => self.check_rt_of_fn(scope, a1, a2, span)?,

            _ => false,
        });
    }

    pub fn rt(&self, scope: &Scope, ty: &Type, span: &dyn Spannable) -> Res<Type> {
        match ty {
            Type::TypeOfVar(name) => {
                let Some((_, t)) = scope.get_combined_with_nonaddressable(name)
                else { return Err(CE {
                    at: span.spanned(),
                    during: "Type resolution - TypeOfVar",
                    error: CEData::NotFoundInScope { needed: name.clone(), scope: scope.clone() },
                    msg: None,
                }) };

                self.rt(scope, &t, span)
            }

            Type::ReturnTypeOf(inner) => {
                let resolved = self.rt(scope, inner, span)?;

                match resolved {
                    Type::Function(f) => {
                        self.rt(scope, &f.ret, span)
                    }

                    _ => todo!("TODO error"),
                }
            }

            Type::Function(f) => {
                let mut rargs = Vec::new();
                for arg in &f.args {
                    rargs.push(self.rt(scope, arg, span)?);
                }

                let rret = self.rt(scope, &f.ret, span)?;
                return Ok(Type::Function(Box::new(FunctionType {
                    ret: rret,
                    generics: Vec::new(),  // TODO generics
                    args: rargs,
                })));
            }

            _ => Ok(ty.clone()),
        }
    }
}
