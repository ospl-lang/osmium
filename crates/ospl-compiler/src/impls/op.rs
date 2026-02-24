use ospl_common::{ast::repr::{Expr, Op}, inst::unoptimized::VMInstruction};

use crate::{BinaryOpEval, Compiler, base::CompilerError};

impl Compiler {
    pub fn binary_op(&mut self, op: &Op, lhs: &Expr, rhs: &Expr, out: &mut Vec<VMInstruction>) -> Result<BinaryOpEval, CompilerError> {
        // we gotta find the type
        let left_eval = self.eval(lhs, out)?;
        let right_eval = self.eval(rhs, out)?;

        // for now just say the type is the LHS's type
        // FIXME: stop doing this!
        let ty = left_eval.ty;

        let left = left_eval.index;
        let right = right_eval.index;

        match op {
            Op::Add => out.push(VMInstruction::AddRegs(left, right)),
            Op::Sub => out.push(VMInstruction::SubRegs(left, right)),
            Op::Div => out.push(VMInstruction::DivRegs(left, right)),
            Op::Mul => out.push(VMInstruction::MulRegs(left, right)),
            Op::Mod => out.push(VMInstruction::ModRegs(left, right)),
        };

        return Ok(BinaryOpEval {
            left, right,
            result: self.top_mut().declarations.next_reg_pre(),
            ty
        })
    }
}
