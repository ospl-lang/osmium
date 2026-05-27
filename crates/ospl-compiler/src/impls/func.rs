use ospl_common::{ast::{Scope, StoreAddress, spanning::Spannable, types::{AType, FunctionType, UType}}, inst::optimized::{Inst, InstBuilder, Opc}};

use crate::{CE, CEData, Compiler, EvalResult, Res, Type, ast::FunctionValue, impls::types::{AtomicResolver, TypeResolver}};

pub struct UtToAt<'a, T>(
    /// The scope
    pub &'a Scope<T>,
);

impl<'a> TypeResolver<UType, AType> for UtToAt<'a, AType> {
    fn resolve(&self, ty: &UType, span: &dyn Spannable) -> Res<Option<AType>> {
        match ty {
            UType::Typeof(var) => {
                let Some(s) = self.0.get_store(&var)
                else { todo!("TODO add error") };

                if let StoreAddress::Generic(g) = s.address {
                    return Ok(Some(AType::Generic(g)))
                }

                return Ok(Some(s.typ.clone()))
            },
            UType::Scope(s) => {
                let mut r: Scope<AType> = Scope::default();
                for key in s.keys_cloned() {
                    let (a, ty) = s.get_combined(&key)
                        .expect("Scope::keys() doesn't work correctly (this is a bug!)");

                    let Some(x) = self.resolve(ty, span)?
                    else { return Ok(None); };

                    r.declare(key, a, x);
                }
                return Ok(Some(AType::Scope(r)));
            }
            UType::PreResolved(e) => {
                return Ok(Some(AType::Resolved(e.clone())))
            },
            UType::List(l) => self.resolve(l, span),
            UType::Function(func) => {
                /* GENERICS */
                let mut generic_constraints: Vec<AType> = Vec::new();
                for i in func.generics.iter() {
                    let t = self.resolve(i, span)?.expect("GenericsResolver is not exhaustive");
                    generic_constraints.push(t);
                }

                /* ARGUMENTS */
                let mut args: Vec<AType> = Vec::new();
                for i in func.args.iter() {
                    let t = self.resolve(i, span)?.expect("GenericsResolver is not exhaustive");
                    args.push(t);
                }

                let ret = self.resolve(&func.ret, span)?.expect("GenericsResolver is not exhaustive");
                return Ok(Some(AType::Resolved(Type::ResolvingFunction(Box::new(FunctionType {
                    args,
                    generics: generic_constraints,
                    ret
                })))))
            },
            UType::AnyScope => unimplemented!(),
            UType::ReturnTypeof(v) => {
                let t = self.resolve(&*v, span)?.expect("GenericsResolver is not exhaustive");
                return Ok(Some(t))
            }
        }
    }
}

pub struct ATypeResolver<'a, T, R: TypeResolver<T, Type>>(pub &'a [T], pub &'a R);

impl<'a, T: Clone, R: TypeResolver<T, Type>> TypeResolver<AType, Type> for ATypeResolver<'a, T, R> {
    fn resolve(&self, ty: &AType, span: &dyn Spannable) -> Res<Option<Type>> {
        tracing::debug!("ATypeResolver resolving {ty:?}");
        match ty {
            AType::Generic(g) => {
                tracing::debug!("ATypeResolver generic #{g}");
                let Some(x) = self.1.resolve(&self.0[*g], span)?
                else { return Ok(None) };

                tracing::debug!("ATypeResolver data for Generic: {x:?}");

                return Ok(Some(x))
            }
            AType::Resolved(g) => return Ok(Some(g.clone())),
            AType::Scope(s) => {
                let mut s2: Scope<Type> = Scope::default();
                for thing in s.keys_cloned() {
                    let store = s.get_store(&thing).expect(".keys_cloned() doesn't work properly");
                    let ty = self.resolve(&store.typ, span)?.expect("ATypeResolver is not exhaustive (this is a bug)");
                    s2.direct_dcl(thing.clone(), store.address.clone(), ty);
                }
                return Ok(Some(Type::Scope(s2)))
            }
        }
    }
}

// pub struct IdentityResolver;

// impl<T: Clone> TypeResolver<T, T> for IdentityResolver {
//     fn resolve(&self, ty: &T, _: &dyn Spannable) -> Res<Option<T>> {
//         return Ok(Some(ty.clone()))
//     }
// }

impl Compiler {
    fn fn_literal_ascope(
        &self,
        func: &FunctionValue,
        span: &dyn Spannable,
    ) -> Res<(Scope<AType>, Vec<usize>)>
    {
        let mut s: Scope<AType> = Scope::default();

        /* CAPTURES */
        let mut capture_indexes = Vec::new();
        for cap in func.captures.iter() {
            let stor = self.stack.top().get_store(cap).unwrap();
            s.direct_dcl(cap.clone(), stor.address.clone(), AType::Resolved(stor.typ.clone()));
            match stor.address {
                StoreAddress::With(w) => capture_indexes.push(w),
                StoreAddress::Generic(_) => {},  // handled at calltime
                StoreAddress::None => {},
            }
        }

        /* GENERICS */
        let mut generic_counter = 0_usize;
        for (g, i) in func.ftype.generics.iter().zip(func.generics.iter()) {
            let ty = UtToAt(&s).resolve(g, span)?.unwrap();
            s.direct_dcl(i.clone(), StoreAddress::Generic(generic_counter), ty.clone());
            generic_counter += 1;
        }

        /* ARGUMENTS */
        let mut args = Vec::new();
        for (a, i) in func.ftype.args.iter().zip(func.args.iter()) {
            let ty = UtToAt(&s).resolve(a, span)?.unwrap();

            let var = s.next_post();
            s.direct_dcl(i.name.clone(), StoreAddress::With(var), ty.clone());

            args.push(ty);
        }

        return Ok((s, capture_indexes))
    }

    fn ascope_to_fn_scope(
        &self,
        func: &FunctionValue,
        ascope: &Scope<AType>,
        span: &dyn Spannable
    ) -> Res<Scope<Type>>
    {
        let mut s: Scope<Type> = Scope::default();
        for key in ascope.keys_cloned() {
            let x = ascope.get_store(&key).expect(".keys_cloned() doesn't work correctly.");
            s.direct_dcl(key, x.address.clone(), match &x.typ {
                AType::Scope(s) => Type::Scope(self.ascope_to_fn_scope(func, &s, span)?),
                AType::Generic(s) => AtomicResolver.resolve(
                    func.ftype.generics.get(*s).expect("fairly certain this can't fail?"),
                    span
                )?.expect("TODO unwrap"),
                AType::Resolved(r) => r.clone()
            });
        }
        return Ok(s)
    }

    pub fn fn_literal(
        &mut self,
        func: &FunctionValue,
        span: &dyn Spannable,
        ob: &mut Vec<Inst>
    ) -> Res<EvalResult>
    {
        /* RESOLVE */
        let (new_scope, generics) = self.fn_literal_ascope(&func, span)?;
        let AType::Resolved(Type::ResolvingFunction(ftype)) =
            UtToAt(&new_scope)
            .resolve(&UType::Function(Box::new(func.ftype.clone())), span)?
            .expect("the thing isn't exhaustive asdasd")
            else { panic!("the thing doesn't work TODO unwrap") };

        // let new_scope = self.ascope_to_fn_scope(func, &new_scope, span)?;
        let new_scope = self.ascope_to_fn_scope(func, &new_scope, span)?;

        /* COMPILE FUNCTION */
        let mut code = Vec::new();
        self.stack.scopes.push(new_scope.clone());
        self.compile_block(&func.block, &mut code)?;
        self.stack.pop();

        /* PUSH */
        ob.push(InstBuilder::new()
            .opcode(Opc::PushFunction)
            .child(code)
            .indexes(&generics)
            .build());

        return Ok(EvalResult {
            address: self.next_var(),
            ty: Type::ResolvingFunction(Box::new(*ftype)),
        })
    }

    pub fn do_call_resolved_fn(
        &mut self,
        span: &dyn Spannable,
        eval: &EvalResult,
        args: &[EvalResult],
        ob: &mut Vec<Inst>,
    ) -> Res<EvalResult>
    {
        let Type::CallableFunction(func) = &eval.ty
        else { return Err(CE {
            at: span.spanned(),
            during: "function call - type check",
            error: CEData::UncallableType { ty: eval.ty.clone() },
            msg: Some("You probably want to specialize it before the call, like `<Option T>.Some(x)`")
        }) };

        // TYPECHECKING...
        if func.args.len() != args.len() {
            // wrong number of args
            return Err(CE {
                at: span.spanned(),
                msg: None,
                during: "function call instance",
                error: CEData::WrongArgCount {
                    expected: func.args.len(),
                    got: args.len()
                }
            });
        }

        let mut new_args = Vec::new();
        for (eval, func_arg) in args.iter().zip(&func.args) {
            if !self.check_type(&eval.ty, func_arg) {
                return Err(CE {
                    at: span.spanned(),
                    msg: Some("perhaps you meant to cast the argument?"),
                    during: "function call instance",
                    error: CEData::MismatchedTypes {
                        expected: crate::TypeExpectation::Exact(func_arg.clone()),
                        got: eval.ty.clone()
                    }
                })
            }
            new_args.push(eval.address);
        }
        let i = InstBuilder::new()
            .opcode(Opc::Call)
            .index(eval.address)  // right here
            .indexes(&new_args)
            .build();

        ob.push(i);

        return Ok(EvalResult {
            address: self.next_var(),
            ty: func.ret.clone()
        })
    }
}