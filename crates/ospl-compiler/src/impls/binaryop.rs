use ospl_common::{ast::ops::{AssignOp, BinaryOp, BinaryOpType}, inst::optimized::{Inst, InstBuilder, Opc}};

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
                msg: Some("Perhaps you meant to cast one type?"),
                error: CEData::MismatchedTypes {
                    expected: crate::TypeExpectation::Exact(left.ty),
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

    pub fn assign_op(
        &mut self,
        b: &AssignOp,
        ob: &mut Vec<Inst>
    ) -> Res<()>
    {
        let left = self.get_lvalue(&b.left, ob)?;
        let right = self.eval(&b.right, ob)?;

        let opc = match &b.kind {
            BinaryOpType::Add       => Opc::Addl,
            BinaryOpType::Subtract  => Opc::Subl,
            BinaryOpType::Multiply  => Opc::Mull,
            BinaryOpType::Divide    => Opc::Divl,
            BinaryOpType::Modulo    => Opc::Modl,
            other => return Err(CE {
                at: Box::new(b.left.clone()),
                error: CEData::InvalidAssignOp { op: other.clone() },
                msg: Some("perhaps you want to use def 'X = X op Y'")
            })
        };

        let inst = InstBuilder::new()
            .opcode(opc)
            .index(left.address)
            .index(right.address)
            .build();

        ob.push(inst);

        return Ok(())
    }
}
