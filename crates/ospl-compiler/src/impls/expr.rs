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
            Expr::Literal(l) => self.literal(l, ob),
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
            Literal::Function(f) => self.fn_literal(f, ob),

            // no idea how to write this without duplicating code.. if anyone knows a cleaner way LMK
            Literal::Int(i) => {
                ob.push(InstBuilder::new().opcode(Opc::PushLiteral).value(ospl_common::inst::RuntimeValue::Int(*i)).build());
                return Ok(EvalResult { address: self.next_var(), ty: Type::Int })
            },
            Literal::Float(f) => {
                ob.push(InstBuilder::new().opcode(Opc::PushLiteral).value(ospl_common::inst::RuntimeValue::Float(*f)).build());
                return Ok(EvalResult { address: self.next_var(), ty: Type::Float })
            },
            Literal::Bool(b) => {
                ob.push(InstBuilder::new().opcode(Opc::PushLiteral).value(ospl_common::inst::RuntimeValue::Bool(*b)).build());
                return Ok(EvalResult { address: self.next_var(), ty: Type::Bool })
            },
            _ => unimplemented!("FIXME")
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

                let i = InstBuilder::new()
                    .opcode(Opc::Property)
                    .index(eval.address)
                    .index(x.address)
                    .build();

                ob.push(i);

                return Ok(EvalResult {
                    address: self.next_var(),
                    ty: x.ty
                })
            },
            // FIXME unwrap
            LV::Variable(var) => {
                let x = EvalResult::from(self.stack.top().get_combined(var).unwrap());
                return Ok(x)
            }
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