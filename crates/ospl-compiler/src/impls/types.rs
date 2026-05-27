use ospl_common::ast::{Scope, spanning::Spannable, types::{FunctionType, Type, UType}};

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
            (Type::Scope(a), Type::Scope(b)) => a == b,
            (Type::CallableFunction(a), Type::CallableFunction(b)) => a == b,

            (Type::ForeignLibrary, Type::ForeignLibrary) => true,

            (Type::ForeignFunction(args1, ret1), Type::ForeignFunction(args2, ret2)) => {
                args1 == args2 && ret1 == ret2
            }

            _ => false,
        }
    }
}

pub trait TypeResolver<U, T> {
    fn resolve(&self, ty: &U, span: &dyn Spannable) -> Res<Option<T>>;
}

pub struct AtomicResolver;

impl TypeResolver<UType, Type> for AtomicResolver {
    fn resolve(&self, ty: &UType, _: &dyn Spannable) -> Res<Option<Type>> {
        match ty {
            UType::PreResolved(p) => return Ok(Some(p.clone())),
            _ => Ok(None),
        }        
    }
}

pub struct DefaultResolver<'a, T>(pub &'a Scope<T>);

impl<'a> TypeResolver<UType, Type> for DefaultResolver<'a, Type> {
    fn resolve(&self, ty: &UType, span: &dyn Spannable) -> Res<Option<Type>> {
        match ty {
            UType::Typeof(name) => {
                let Some(s) = self.0.get_store(name)
                else { return Err(CE {
                    at: span.spanned(),
                    during: "Type resolution - TypeOfVar",

                    // TODO harcoding
                    error: CEData::NotFoundInScope { needed: name.clone(), scope: Scope::default() },
                    msg: None,
                }) };

                // let t = match self.resolve(ty, span) {
                //     Some(Ok(x)) => x,
                //     e @ Some(Err(_)) => return e,
                //     None => return None
                // };

                return Ok(Some(s.typ.clone()))
            }

            UType::ReturnTypeof(inner) => {
                let Some(resolved) = self.resolve(inner, span)?
                else { return Err(CE {
                    at: span.spanned(),
                    during: "Type resolution - DefaultResolver - ReturnTypeof",
                    msg: None,
                    error: CEData::UnresolvableUType { t: Box::new(*inner.clone()) },
                })};

                match resolved {
                    Type::CallableFunction(f) => self.resolve(&f.ret.into(), span),
                    _ => todo!("TODO error"),
                }
            }

            UType::Function(f) => {
                let mut rargs = Vec::new();
                for arg in &f.args {
                    rargs.push(self.resolve(arg, span)?.expect("todo unwrap"));
                }

                let rret = self.resolve(&f.ret, span)?.expect("todo unwrap");
                return Ok(Some(Type::CallableFunction(Box::new(FunctionType {
                    ret: rret,
                    generics: Vec::new(),  // TODO generics
                    args: rargs,
                }))));
            },

            _ => return AtomicResolver.resolve(ty, span),

            // other => return Err(CE {
            //     at: span.spanned(),
            //     during: "Type resolution",
            //     error: CEData::UnresolvableType { t: other.clone() },
            //     msg: Some("Report this one, it's probably a compiler bug. (note to devs: you need to implement this type's resolution in the function it's used)")
            // })
        }
    }
}
