use ospl_common::{
    ast::repr::{Expr, FunctionData, Type},
    inst::{RuntimeFunctionData, unoptimized::VMInstruction},
};

use crate::{Compiler, base::{CompilerError, Eval, Res}};

impl Compiler {
    pub fn compile_fn(&mut self, fd: &FunctionData) -> Res<RuntimeFunctionData> {
        let mut translated = Vec::with_capacity(fd.code.len());

        self.new_scope_parent()?;
        for line in &fd.code {
            self.compile_stmt(line, &mut translated)?;
        }
        self.pop_scope()?;

        return Ok(RuntimeFunctionData {
            code: translated
        })
    }

    pub fn call_fn(
        &mut self,
        func: &Expr,
        args: &[Expr],
        out: &mut Vec<VMInstruction>
    ) -> Res<Eval> {
        // PersonP -- do not want to do this clone. But I think we have to
        let store = self.eval(func, out)?;

        let Type::Function(fd) = &store.ty
            else { return Err(CompilerError::TypeNotCallable) };

        // validate
        let mut indexes = Vec::new();
        for arg in args {
            let e = self.eval(arg, out)?;
            indexes.push(e.index);
        }

        // preform call
        self.new_scope_parent()?;
        out.push(VMInstruction::Call(store.index, indexes));

        return Ok(Eval {
            index: self.top_mut()?.next_pre(),
            ty: (*fd.ret).clone(),
        })
    }

    pub fn ret(
        &mut self,
        e: &Expr,
        out: &mut Vec<VMInstruction>
    ) -> Res<()> {
        let data = self.eval(e, out)?;
        out.push(VMInstruction::Ret(data.index));

        return Ok(())
    }
}
