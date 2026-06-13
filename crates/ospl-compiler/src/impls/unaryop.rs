use ospl_common::{ast::{Type, ops::{UnaryOp, UnaryOpType}, spanning::Spannable}, inst::optimized::{Inst, InstBuilder, Opc}};

use crate::{CE, CEData, Compiler, EvalResult, Res};

impl Compiler {
    pub fn unary_op(
        &mut self,
        u: &UnaryOp,
        ob: &mut Vec<Inst>,
        span: &dyn Spannable,
    ) -> Res<EvalResult>
    {
        let eval = self.eval(&u.expr, ob)?;

        let new_ty = match (&eval.ty, &u.kind) {
            (ty, UnaryOpType::LogicNot) => ty.clone(),
            (Type::List(lty), UnaryOpType::Decrement) => *lty.clone(),
            (Type::Nominal(_, t), UnaryOpType::Atsign) => {
                return Ok(EvalResult {
                    address: eval.address,
                    ty: *t.clone()
                })
            }
            (ty, op) => return Err(CE {
                at: span.spanned(),
                during: "Unary operator - type check",
                error: CEData::InvalidUnaryOpForType { op: op.clone(), ty: ty.clone() },
                msg: None
            })
        };

        ob.push(InstBuilder::new()
            .opcode(unary_op_to_opc(&u.kind))
            .index(eval.address)
            .build());

        return Ok(EvalResult {
            address: self.next_var(),
            ty: new_ty,
        })
    }
}

pub fn unary_op_to_opc(t: &UnaryOpType) -> Opc {
    return match t {
        UnaryOpType::Decrement => Opc::Decrement,
        UnaryOpType::Increment => Opc::Increment,
        UnaryOpType::LogicNot => Opc::Neg,
        UnaryOpType::Copy => Opc::Copy,
        _ => unreachable!()
    }
}
