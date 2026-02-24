use ospl_common::{ast::repr::{Expr, LValue, StaticValue}, inst::{RuntimeStaticValue, RuntimeStructureData, unoptimized::VMInstruction}};

use crate::{Compiler, Eval, base::{CompilerError, Local}};

mod decl;
mod op;
mod func;
mod stmt;

impl Compiler {
    pub fn get_lvalue_ref(&self, lv: &LValue) -> &Local {
        return match lv {
            LValue::Var(v) => self.top().declarations.get(v).unwrap()
        }
    }

    pub fn get_lvalue_reg(&self, lv: &LValue) -> usize {
        return self.get_lvalue_ref(lv).register
    }

    pub fn static_to_runtime(&mut self, value: StaticValue) -> Result<RuntimeStaticValue, CompilerError> {
        return match value {
            StaticValue::Float(f) => Ok(RuntimeStaticValue::Float(f)),
            StaticValue::Int(i) => Ok(RuntimeStaticValue::Int(i)),
            StaticValue::Nul => Ok(RuntimeStaticValue::Nul),

            StaticValue::Structure(_) => {
                return Ok(RuntimeStaticValue::Structure(RuntimeStructureData {
                    items: Vec::new()
                }))
            },
            StaticValue::Function(fd) => Ok(self.compile_fn(fd)?),
        }
    }

    /// Returns index of newly evaluated expr
    pub fn eval(&mut self, expr: &Expr, out: &mut Vec<VMInstruction>) -> Result<Eval, CompilerError> {
        return match expr {
            Expr::BinaryOp { op, lhs, rhs } => Ok(self.binary_op(op, lhs, rhs, out)?.into()),
            Expr::LValue(lv) => Ok(Eval::from(self.get_lvalue_ref(&lv).clone())),
            Expr::StaticLiteral(st) => Ok(Eval::with_index({
                let rt = self.static_to_runtime(st.clone())?;
                out.push(VMInstruction::PushLiteral(rt));

                self.top().declarations.get_reg()
            })),
            Expr::Call(lv, args) => Ok(self.fn_call(lv, args, out)?),
        }
    }
}