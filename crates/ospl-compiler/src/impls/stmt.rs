use ospl_common::{ast::repr::{Expr, Stmt}, inst::unoptimized::VMInstruction};

use crate::{Compiler, base::CompilerError};

impl Compiler {
    pub fn return_stmt(&mut self, expr: &Expr, out: &mut Vec<VMInstruction>) -> Result<(), CompilerError> {
        let eval = self.eval(expr, out)?;
        out.push(VMInstruction::Ret(eval.index));

        return Ok(())
    }

    pub fn compile_stmt(&mut self, stmt: Stmt, out: &mut Vec<VMInstruction>) -> Result<(), CompilerError> {
        match stmt {
            Stmt::Declare { lhs, rhs } => { match rhs {
                Some(e) => self.declare_lvalue_init(lhs, e, out),
                None => self.declare_lvalue_uninit(lhs, out),
            }?; },
            Stmt::Assign { lhs, rhs } => { self.assign_lvalue(lhs, &rhs, out)?; },
            Stmt::Return(r) => { self.return_stmt(&r, out)?; }
            _ => {}
        };

        return Ok(())
    }

    pub fn compile_stmt_list(&mut self, list: Vec<Stmt>, out: &mut Vec<VMInstruction>) -> Result<(), CompilerError> {
        for stmt in list {
            self.compile_stmt(stmt, out)?;
        }

        return Ok(())
    }

}