use ospl_common::{ast::{Type, ops::{UnaryOp, UnaryOpType}}, inst::optimized::{Inst, InstBuilder, Opc}};

use crate::{Compiler, EvalResult, Res};

impl Compiler {
    pub fn unary_op(
        &mut self,
        u: &UnaryOp,
        ob: &mut Vec<Inst>
    ) -> Res<EvalResult>
    {
        let eval = self.eval(&u.expr, ob)?;

        let new_ty = match (&eval.ty, &u.kind) {
            (Type::List(lty), UnaryOpType::Increment) => *lty.clone(),
            (Type::Scope(_), UnaryOpType::Increment) => {
                unimplemented!();
                // return Ok(EvalResult {
                //     address: self.next_var(),
                //     ty: Type::List(Box::new(Type::Str))
                // })
            },
            _ => panic!("TODO - add error for invalid unary op type")
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
        UnaryOpType::Atsign => unimplemented!("atsign (@) operation is not implemented")
    }
}
