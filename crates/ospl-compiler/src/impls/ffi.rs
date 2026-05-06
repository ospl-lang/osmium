use ospl_common::{ast::{Expression, LValue, Type}, inst::optimized::{Inst, InstBuilder, Opc}};

use crate::{Compiler, EvalResult, Res};

impl Compiler {
    pub fn ffi_load(
        &mut self,
        lib_path: &Expression,
        ob: &mut Vec<Inst>
    ) -> Res<EvalResult>
    {
        let lib = self.eval(lib_path, ob)?;
        if lib.ty != Type::Str {
            /* error */
        }

        ob.push(InstBuilder::new()
            .opcode(Opc::FFILoadLib)
            .index(lib.address)
            .build());

        return Ok(EvalResult {
            address: self.next_var(),
            ty: Type::ForeignLibrary,
        })
    }

    pub fn ffi_func(
        &mut self,
        lib: &LValue,
        func_name: &Expression,
        ret_type: usize,
        types: &[usize],
        ob: &mut Vec<Inst>
    ) -> Res<EvalResult>
    {
        let lib = self.get_lvalue(lib, ob)?;
        let func_name = self.eval(func_name, ob)?;

        ob.push(InstBuilder::new()
            .opcode(Opc::FFILoadFn)
            .index(lib.address)
            .index(func_name.address)
            .index(ret_type)
            .indexes(types)
            .build());

        return Ok(EvalResult {
            address: self.next_var(),
            ty: Type::ForeignFunction,
        })
    }

    pub fn ffi_call(
        &mut self,
        func: &LValue,
        args: &[Expression],
        ob: &mut Vec<Inst>
    ) -> Res<EvalResult>
    {
        let func = self.get_lvalue(func, ob)?;
        let mut new_args = Vec::new();
        for arg in args {
            new_args.push(self.eval(arg, ob)?.address);
        }

        ob.push(InstBuilder::new()
            .opcode(Opc::FFICall)
            .index(func.address)
            .indexes(&new_args)
            .build());

        return Ok(EvalResult {
            address: self.next_var(),
            ty: Type::ForeignFunction,
        })
    }
}