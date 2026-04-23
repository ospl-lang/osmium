use ospl_common::{ast::repr::{Expr, Op}, inst::unoptimized::VMInstruction};

use crate::{Compiler, base::{CompilerError, Eval, Res}};

impl Compiler {
    pub fn binaryop(
        &mut self,
        op: &Op,
        lhs: &Expr,
        rhs: &Expr,
        out: &mut Vec<VMInstruction>,
    ) -> Res<Eval>
    {
        // just assume it's primitive for now
        
        let lvalue = self.eval(lhs, out)?;
        let rvalue = self.eval(rhs, out)?;
        if lvalue.ty != rvalue.ty {
            return Err(CompilerError::MismatchedType {
                expected: lvalue.ty,
                got: rvalue.ty
            })
        }

        match op {
            Op::Add => out.push(VMInstruction::AddRegs(lvalue.index, rvalue.index)),
            Op::Sub => out.push(VMInstruction::SubRegs(lvalue.index, rvalue.index)),
            Op::Mul => out.push(VMInstruction::MulRegs(lvalue.index, rvalue.index)),
            Op::Div => out.push(VMInstruction::DivRegs(lvalue.index, rvalue.index)),
            Op::Mod => out.push(VMInstruction::ModRegs(lvalue.index, rvalue.index)),
        }

        // output do be like the next one
        return Ok(Eval {
            index: self.top_mut()?.next_pre(),
            ty: lvalue.ty,
        })
    }
}