use ospl_common::inst::optimized::{Inst, InstBuilder, Opc};

use crate::{Compiler, EvalResult, Store, Type, ast::{Expr, LV, LValue, Literal}};

impl Compiler {
    pub fn eval(
        &mut self,
        expr: &crate::ast::Expression,
        ob: &mut Vec<Inst>
    ) -> EvalResult
    {
        match &*expr.inner {
            Expr::Literal(l) => return self.literal(l, ob),
            Expr::Call(func, args) => {
                let f = self.eval(func, ob);
                let Type::Function(func) = f.ty
                    else { unimplemented!() };  // FIXME: use result

                // TYPECHECKING...
                if func.args.len() != args.len() {
                    // wrong number of args
                    // FIXME: actually throw error
                }
                
                let mut new_args = Vec::new();
                for arg in args {
                    let eval = self.eval(arg, ob);
                    new_args.push(eval.address);
                }

                ob.push(InstBuilder::new()
                    .opcode(Opc::Call)
                    .index(f.address)
                    .indexes(&new_args)
                    .build());

                return EvalResult {
                    address: self.next_var(),
                    ty: func.ret.clone()
                }
            },
            Expr::LValue(lv) => {
                let store = self.get_lvalue(lv, ob);
                return EvalResult {
                    address: store.var,
                    ty: store.ty.clone()
                }
            }
        }
    }

    pub fn literal(
        &mut self,
        l: &Literal,
        ob: &mut Vec<Inst>
    ) -> EvalResult
    {
        match l {
            Literal::Function(f) => return self.fn_literal(f, ob),
            Literal::Int(i) => {
                ob.push(InstBuilder::new().opcode(Opc::PushLiteral).value(ospl_common::inst::RuntimeValue::Int(*i)).build());
                return EvalResult { address: self.next_var(), ty: Type::Int }
            }
            // other => self.literal_generic(i),
        }
    }

    pub fn get_lvalue(
        &mut self,
        lv: &LValue,
        ob: &mut Vec<Inst>
    ) -> &Store
    {
        match &*lv.inner {
            LV::Property(lv2, var) => {
                let eval = self.get_lvalue(lv2, ob);
                let x = match &eval.ty {
                    Type::Scope(s) => s.map.get(&*var),
                    _ => None
                }.unwrap();

                ob.push(InstBuilder::new()
                    .opcode(Opc::Property)
                    .index(eval.var)
                    .index(x.var)
                    .build());

                return x
            },
            LV::Variable(v) => return self.stack.top().map.get(v)
                .unwrap()// FIXME
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