use crate::Compiler;
use ospl_common::ast::repr::{Expr, LValue, StaticValue, Stmt};
use ospl_common::inst::VMInstruction;

impl Compiler {
    pub fn declare_lvalue_uninit(&mut self, lvalue: LValue) -> usize {
        return self.declare_lvalue_init(lvalue, Expr::StaticLiteral(StaticValue::Nul));
    }

    pub fn declare_lvalue_init(&mut self, lvalue: LValue, rvalue: Expr) -> usize {
        match lvalue {
            LValue::Var(v) => {
                // evaluate the rvalue
                let eval = self.eval(&rvalue);

                let decl_idx = self.scope.declare(v, eval.ty.data);

                assert_eq!(eval.index, decl_idx);

                // already declared in this `eval`
                return eval.index;
            }
        }
    }

    pub fn assign_lvalue(&mut self, lvalue: LValue, rvalue: &Expr) {
        let left_index = self.get_lvalue_index(&lvalue);

        // OPTIMIZE: can be optimized.
        // 
        // We're likely going to end up pushing new data, even when it's
        // not needed!
        //
        // Two routes can be taken to optimize this:
        //  a. Detect when new data is pushed and optimize accordingly.
        //  b. Guarantee `eval(...)` doesn't push new data. (likely infeasable)
        let eval = self.eval(rvalue);
        
        self.code.push(VMInstruction::AssignCopy {
            reg: left_index,
            new: eval.index
        });
    }

    pub fn compile_stmt(&mut self, stmt: Stmt) {
        match stmt {
            Stmt::Declare { lhs, rhs } => { match rhs {
                Some(e) => self.declare_lvalue_init(lhs, e),
                None => self.declare_lvalue_uninit(lhs),
            }; },
            _ => {}
        };
    }
}