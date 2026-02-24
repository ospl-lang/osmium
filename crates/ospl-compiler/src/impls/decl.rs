use crate::base::CompilerError;
use crate::Compiler;
use ospl_common::ast::repr::{Expr, LValue, StaticValue};
use ospl_common::inst::unoptimized::VMInstruction;

impl Compiler {
    pub fn declare_lvalue_uninit(&mut self, lvalue: LValue, out: &mut Vec<VMInstruction>) -> Result<usize, CompilerError> {
        return self.declare_lvalue_init(lvalue, Expr::StaticLiteral(StaticValue::Nul), out);
    }

    pub fn declare_lvalue_init(&mut self, lvalue: LValue, rvalue: Expr, out: &mut Vec<VMInstruction>) -> Result<usize, CompilerError> {
        match lvalue {
            LValue::Var(v) => {
                // evaluate the rvalue
                let eval = self.eval(&rvalue, out)?;

                let decl_idx = self.top_mut().declare(v, eval.ty.data);

                assert_eq!(eval.index, decl_idx);

                // already declared in this `eval`
                return Ok(eval.index);
            }
        }
    }

    pub fn assign_lvalue(&mut self, lvalue: LValue, rvalue: &Expr, out: &mut Vec<VMInstruction>) -> Result<(), CompilerError> {
        let left_index = self.get_lvalue_reg(&lvalue);

        // OPTIMIZE: can be optimized.
        // 
        // We're likely going to end up pushing new data, even when it's
        // not needed!
        //
        // Two routes can be taken to optimize this:
        //  a. Detect when new data is pushed and optimize accordingly.
        //  b. Guarantee `eval(...)` doesn't push new data. (likely infeasable)
        let eval = self.eval(rvalue, out)?;
        
        out.push(VMInstruction::AssignCopy {
            reg: left_index,
            new: eval.index
        });

        return Ok(())
    }
}
