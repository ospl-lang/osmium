use ospl_common::inst::optimized::{Inst, InstBuilder, Opc};

use crate::{Compiler, EvalResult, Res, Type, ast::{Expr, LV, LValue, Literal}};

impl Compiler {
    pub fn eval(
        &mut self,
        expr: &crate::ast::Expression,
        ob: &mut Vec<Inst>
    ) -> Res<EvalResult>
    {
        match &*expr.inner {
            Expr::Literal(l) => return self.literal(l, ob),
            Expr::Call(func, args) => self.do_call(func, args, ob),
            Expr::BinaryOp(b) => self.binary_op(b, ob),
            Expr::LValue(lv) => {
                let store = self.get_lvalue(lv, ob)?;
                return Ok(EvalResult {
                    address: store.address,
                    ty: store.ty.clone()
                })
            }
        }
    }

    pub fn literal(
        &mut self,
        l: &Literal,
        ob: &mut Vec<Inst>
    ) -> Res<EvalResult>
    {
        match l {
            Literal::Function(f) => return self.fn_literal(f, ob),
            Literal::Int(i) => {
                ob.push(InstBuilder::new().opcode(Opc::PushLiteral).value(ospl_common::inst::RuntimeValue::Int(*i)).build());
                return Ok(EvalResult { address: self.next_var(), ty: Type::Int })
            }
            // other => self.literal_generic(i),
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
                let x = match &eval.ty {
                    Type::Scope(s) => EvalResult::from(s.get_combined(var).unwrap()),  // FIXME unwrap
                    _ => unimplemented!()
                };

                ob.push(InstBuilder::new()
                    .opcode(Opc::Property)
                    .index(eval.address)
                    .index(x.address)
                    .build());

                return Ok(x)
            },
            // FIXME unwrap
            LV::Variable(var) => return Ok(EvalResult::from(self.stack.top().get_combined(var).unwrap()))
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