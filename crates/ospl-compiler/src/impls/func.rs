use ospl_common::{ast::{Expression, Stmt}, inst::optimized::{Inst, InstBuilder, Opc}};

use crate::{CompErr, Compiler, EvalResult, Res, Type, ast::FunctionValue, impls::stmt::Control};

impl Compiler {
    /// Unsafe if called more than once on the same FunctionValue.
    pub fn fn_literal(
        &mut self,
        func: &FunctionValue,
        ob: &mut Vec<Inst>
    ) -> Res<EvalResult>
    {
        let mut insts = Vec::<Inst>::new();
        let mut captures = Vec::new();

        self.stack.push();

        for stmt in &func.block {
            // silly way of implementing the `use` statement for closures.
            if let Stmt::ClosureUse(upper_ident, upper_type) = &*stmt.inner {
                let Some(x) = self.stack.scopes.get(self.stack.scopes.len() - 1)
                    else { return Err(CompErr::NoScopeToCapture) };
                
                let Some(x) = x.get_combined(&upper_ident)
                    else { return Err(CompErr::NotFoundInScope { needed: upper_ident.to_string() } )};
                
                if x.1 != upper_type {
                    return Err(CompErr::MismatchedTypes {
                        expected: upper_type.to_owned(),
                        got: x.1.to_owned()
                    })
                }

                captures.push(x.0);
                continue;
            }

            match self.compile_stmt(stmt, &mut insts)? {
                Control::Default => {},
                _ => {}
            }
        };

        let inst = InstBuilder::new()
            .opcode(Opc::PushFunction)
            .child(insts)
            .indexes(&captures)
            .build();

        ob.push(inst);

        self.stack.pop();

        return Ok(EvalResult {
            address: self.next_var(),
            ty: Type::Function(Box::new(func.ftype.clone()))
        })
    }

    pub fn do_call(
        &mut self,
        func: &Expression,
        args: &[Expression],
        ob: &mut Vec<Inst>,
    ) -> Res<EvalResult>
    {
        let f = self.eval(func, ob)?;
        let Type::Function(func) = f.ty
            else { unimplemented!() };  // FIXME: use result

        // TYPECHECKING...
        if func.args.len() != args.len() {
            // wrong number of args
            return Err(CompErr::WrongArgCount {
                expected: func.args.len(),
                got: args.len()
            })
        }

        let mut new_args = Vec::new();
        for (arg, func_arg) in args.iter().zip(func.args) {
            let eval = self.eval(arg, ob)?;
            if eval.ty != func_arg {
                return Err(CompErr::MismatchedTypes {
                    expected: func_arg,
                    got: eval.ty
                })
            }
            new_args.push(eval.address);
        }

        let i = InstBuilder::new()
            .opcode(Opc::Call)
            .index(f.address)  // right here
            .indexes(&new_args)
            .build();

        ob.push(i);

        return Ok(EvalResult {
            address: self.next_var(),
            ty: func.ret.clone()
        })
    }
}