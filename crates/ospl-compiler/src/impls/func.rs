use ospl_common::{ast::repr::{Expr, FunctionData}, inst::{RuntimeFunctionData, RuntimeStaticValue, unoptimized::VMInstruction}};

use crate::{Compiler, Eval, base::{CompilerError, EvalError, TypeData}};

impl Compiler {
    pub fn compile_fn(&mut self, fd: FunctionData) -> Result<RuntimeStaticValue, CompilerError> {
        let mut out = Vec::new();
        for stmt in fd.code {
            self.compile_stmt(stmt, &mut out)?;
        };

        return Ok(RuntimeStaticValue::Function(RuntimeFunctionData {
            code: out
        }))
    }

    pub fn fn_call(&mut self, left: &Expr, args: &Vec<Expr>, out: &mut Vec<VMInstruction>) -> Result<Eval, CompilerError> {
        // a bunch of type correctness checks
        let left = self.eval(left, out)?;
        let TypeData::Function { params, return_type, .. } = left.ty.data
            else { return Err(CompilerError::Eval(EvalError::CalledUncallable)) };

        // ensure right arg number
        if args.len() != params.len() {
            return Err(CompilerError::Eval(EvalError::WrongArgCount {
                expected: params.len(),
                got: args.len()
            }))
        }

        // ensure correct arg types, and convert
        let mut args_indexes = Vec::new();
        for (arg, param) in args.iter().zip(params) {
            let ev = self.eval(arg, out)?;

            if ev.ty != param {
                return Err(CompilerError::Eval(EvalError::WrongArgType {
                    expected: param,
                    found: ev.ty
                }));
            }

            args_indexes.push(ev.index);
        }

        // emit actual call
        out.push(VMInstruction::Call(left.index, args_indexes));

        // we'll get a return on the next slot, so let's just declare this
        let eval = Eval::new(
            self.top_mut().declarations.next_reg_post(),
            *return_type,
        );

        return Ok(eval)
    }
}
