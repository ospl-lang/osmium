use ospl_common::{ast::ops::{BinaryOp, BinaryOpType}, inst::optimized::{Inst, InstBuilder, Opc}};

use crate::{CompErr, Compiler, EvalResult, Res};

impl Compiler {
    pub fn binary_op(
        &mut self,
        b: &BinaryOp,
        ob: &mut Vec<Inst>
    ) -> Res<EvalResult>
    {
        let left = self.eval(&b.left, ob)?;
        let right = self.eval(&b.right, ob)?;
        if left.ty != right.ty {
            return Err(CompErr::MismatchedTypes {
                expected: left.ty,
                got: right.ty
            })
        }

        let opc = match b.kind {
            BinaryOpType::Add       => Opc::Add,
            BinaryOpType::Subtract  => Opc::Sub,
            BinaryOpType::Multiply  => Opc::Mul,
            BinaryOpType::Divide    => Opc::Div,
            BinaryOpType::Modulo    => Opc::Mod,
        };

        let inst = InstBuilder::new()
            .opcode(opc)
            .index(left.address)
            .index(right.address)
            .build();

        ob.push(inst);

        return Ok(EvalResult {
            address: self.next_var(),
            ty: left.ty.clone()
        })
    }
}
