use ospl_common::{ast::repr::{Expr, Op}, inst::VMInstruction};

use crate::{BinaryOpEval, Compiler};

impl Compiler {
    pub fn binary_op(&mut self, op: &Op, lhs: &Expr, rhs: &Expr) -> BinaryOpEval {
        let left = self.eval(lhs).index;
        let right = self.eval(rhs).index;
        match op {
            Op::Add => self.code.push(VMInstruction::AddRegs(left, right)),
            Op::Sub => self.code.push(VMInstruction::SubRegs(left, right)),
            Op::Div => self.code.push(VMInstruction::DivRegs(left, right)),
            Op::Mul => self.code.push(VMInstruction::MulRegs(left, right)),
            Op::Mod => self.code.push(VMInstruction::ModRegs(left, right)),
        };

        return BinaryOpEval {
            left, right,
            result: self.scope.declarations.next_reg_pre()
        }
    }
}
