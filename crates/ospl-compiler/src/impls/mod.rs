use ospl_common::{ast::repr::{Expr, LValue, StaticValue}, inst::VMInstruction};

use crate::{Compiler, Eval, base::Type};

mod decl;
mod op;
mod expr;

impl Compiler {
    pub fn get_lvalue_index(&self, lv: &LValue) -> usize {
        return match lv {
            LValue::Var(v) => self.scope.declarations.get(v).unwrap().register
        }
    }

    /// Small helper function
    fn push_static_literal(&mut self, value: &StaticValue) -> usize {
        self.code.push(VMInstruction::PushLiteral(value.clone()));
        return self.scope.declarations.get_reg()
    }

    /// Returns index of newly evaluated expr
    pub fn eval(&mut self, expr: &Expr) -> Eval {
        let index = match expr {
            Expr::BinaryOp { op, lhs, rhs } => self.binary_op(op, lhs, rhs).result,
            Expr::LValue(lv) => self.get_lvalue_index(&lv),
            Expr::StaticLiteral(st) => self.push_static_literal(st),
            _ => unimplemented!()
        };

        return Eval { ty: Type::default(), index }
    }
}