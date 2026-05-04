use ospl_common::{ast::ops::{BinaryOp, BinaryOpType}, inst::optimized::{Inst, InstBuilder, Opc}};

use crate::{CE, CEData, Compiler, EvalResult, Res};

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
            return Err(CE {
                at: Box::new(b.left.clone()),
                hint_msg: Some("Perhaps you meant to cast one type?"),
                error: CEData::MismatchedTypes {
                    expected: left.ty,
                    got: right.ty
                }
            })
        }

        let opc = match b.kind {
            BinaryOpType::Add       => Opc::Add,
            BinaryOpType::Subtract  => Opc::Sub,
            BinaryOpType::Multiply  => Opc::Mul,
            BinaryOpType::Divide    => Opc::Div,
            BinaryOpType::Modulo    => Opc::Mod,
            BinaryOpType::Equals    => Opc::Eq,
            BinaryOpType::NotEquals => Opc::Neq,
            BinaryOpType::Gt        => Opc::Gt,
            BinaryOpType::Lt        => Opc::Lt,
            BinaryOpType::Ge        => Opc::Gte,
            BinaryOpType::Le        => Opc::Lte,
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
