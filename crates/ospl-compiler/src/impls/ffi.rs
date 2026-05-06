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

        let ret = typeno_to_type(ret_type);

        let arg_types: Vec<Type> = types.iter().map(|i| typeno_to_type(*i)).collect();

        return Ok(EvalResult {
            address: self.next_var(),
            ty: Type::ForeignFunction(arg_types, Box::new(ret)),
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
        let Type::ForeignFunction(_, r) = &func.ty
            else { panic!("can't call a foreign function without foreign keyword") };

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
            ty: *r.clone()
        })
    }
}

fn typeno_to_type(i: usize) -> Type {
    return match i {
        /* u8 */  0 => Type::Address,
        /* i8 */  1 => Type::Int,
        /* u16 */ 2 => Type::Address,
        /* i16 */ 3 => Type::Int,
        /* u32 */ 4 => Type::Address,
        /* i32 */ 5 => Type::Int,
        /* u64 */ 6 => Type::Address,
        /* i64 */ 7 => Type::Int,
        /* f32 */ 8 => Type::Float,
        /* f64 */ 9 => Type::Float,
        /* void */ 10 => Type::Nul,
        /* ptr */  11 => Type::Address,
        /* err */ _ => unimplemented!("unknown FFI typeno"),
    }
}