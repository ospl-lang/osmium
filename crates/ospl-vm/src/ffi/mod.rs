use std::ffi::{c_void, CString};

use libffi::middle::{arg, Arg, CodePtr, Cif, Type};
use libloading::Library;
use ospl_common::inst::{RT, RuntimeValue, make};

#[derive(Debug, Clone)]
pub struct ForeignFunction {
    pub symbol: String,
    pub symbol_ptr: CodePtr,
    pub arg_types: Vec<String>,
    pub return_type: String,
    pub cif: Cif,
}

pub type LibHandle = u32;
pub type FuncHandle = u32;

#[derive(Debug)]
pub struct FfiRegistry {
    libraries: Vec<Option<Library>>,
    functions: Vec<Option<ForeignFunction>>,
}

impl Default for FfiRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl FfiRegistry {
    pub fn new() -> Self {
        Self {
            libraries: Vec::new(),
            functions: Vec::new(),
        }
    }

    pub fn load_library(&mut self, path: &str) -> Result<LibHandle, String> {
        let lib = unsafe { Library::new(path) }
            .map_err(|e| format!("Failed to load library '{}': {}", path, e))?;

        let handle = self.libraries.len() as LibHandle;
        self.libraries.push(Some(lib));
        Ok(handle)
    }

    pub fn get_library(&self, handle: LibHandle) -> Option<&Library> {
        self.libraries.get(handle as usize)?.as_ref()
    }

    // pub fn register_function(
    //     &mut self,
    //     lib: LibHandle,
    //     symbol_name: &str,
    //     arg_types: Vec<String>,
    //     return_type: String,
    // ) -> Result<FuncHandle, String> {
    //     let library = self
    //         .get_library(lib)
    //         .ok_or_else(|| format!("Invalid library handle {}", lib))?;

    //     let symbol_ptr = unsafe {
    //         *library
    //             .get::<*const c_void>(symbol_name.as_bytes())
    //             .map_err(|e| format!("Failed to load symbol '{}': {}", symbol_name, e))?
    //     };

    //     let cif = build_cif(&arg_types, &return_type)
    //         .map_err(|e| format!("Failed to build CIF: {}", e))?;

    //     let func = ForeignFunction {
    //         symbol: symbol_name.to_string(),
    //         symbol_ptr: CodePtr::from_ptr(symbol_ptr),
    //         arg_types,
    //         return_type,
    //         cif,
    //     };

    //     let handle = self.functions.len() as FuncHandle;
    //     self.functions.push(Some(func));

    //     Ok(handle)
    // }

    pub fn register_function(
        &mut self,
        lib: LibHandle,
        symbol_name: &str,
        arg_types: Vec<String>,
        return_type: String,
    ) -> Result<FuncHandle, String> {
        let library = self
            .get_library(lib)
            .ok_or_else(|| format!("Invalid library handle {}", lib))?;

        let symbol_ptr = unsafe {
            *library
                .get(symbol_name.as_bytes())
                .map_err(|e| format!("Failed to load symbol '{}': {}", symbol_name, e))?
        };

        let cif = build_cif(&arg_types, &return_type)
            .map_err(|e| format!("Failed to build CIF: {}", e))?;

        let func = ForeignFunction {
            symbol: symbol_name.to_string(),
            symbol_ptr: CodePtr::from_ptr(symbol_ptr),
            arg_types,
            return_type,
            cif,
        };

        let handle = self.functions.len() as FuncHandle;
        self.functions.push(Some(func));

        Ok(handle)
    }

    pub fn get_function(&self, handle: FuncHandle) -> Option<&ForeignFunction> {
        self.functions.get(handle as usize)?.as_ref()
    }

    pub fn get_function_mut(&mut self, handle: FuncHandle) -> Option<&mut ForeignFunction> {
        self.functions.get_mut(handle as usize)?.as_mut()
    }
}

fn build_cif(arg_types: &[String], return_type: &str) -> Result<Cif, &'static str> {
    let mut args = Vec::new();
    for t in arg_types {
        args.push(map_type(t)?);
    }

    let ret = map_type(return_type)?;
    Ok(Cif::new(args.into_iter(), ret))
}

pub fn map_type(name: &str) -> Result<Type, &'static str> {
    match name {
        "void" => Ok(Type::void()),
        "bool" => Ok(Type::u8()),
        "u8" => Ok(Type::u8()),
        "i8" => Ok(Type::i8()),
        "u16" => Ok(Type::u16()),
        "i16" => Ok(Type::i16()),
        "u32" => Ok(Type::u32()),
        "i32" => Ok(Type::i32()),
        "u64" => Ok(Type::u64()),
        "i64" => Ok(Type::i64()),
        "f32" => Ok(Type::f32()),
        "f64" => Ok(Type::f64()),
        "usize" => Ok(Type::usize()),
        "ptr" | "pointer" | "cstr" => Ok(Type::pointer()),
        _ => Err("unsupported type"),
    }
}

pub fn type_number_to_string(num: usize) -> &'static str {
    match num {
        0 => "u8",
        1 => "i8",
        2 => "u16",
        3 => "i16",
        4 => "u32",
        5 => "i32",
        6 => "u64",
        7 => "i64",
        8 => "f32",
        9 => "f64",
        10 => "void",
        11 => "ptr",
        other => panic!("unknown FFI type number {other}. Valid ones are from 0-11")
    }
}

pub fn call_foreign_function(
    function: &ForeignFunction,
    args: &[RuntimeValue],
) -> Result<RuntimeValue, String> {
    if args.len() != function.arg_types.len() {
        return Err(format!(
            "Argument count mismatch: expected {}, got {} | args: {:?}",
            function.arg_types.len(),
            args.len(),
            args,
        ));
    }

    let mut arg_values: Vec<Box<dyn RawValueHolder>> = Vec::new();
    let mut raw_args: Vec<Arg> = Vec::new();

    for (value, ty) in args.iter().zip(function.arg_types.iter()) {
        let holder = box_value(value.clone(), ty)?;
        raw_args.push(arg(holder.raw_ptr()));
        arg_values.push(holder);
    }

    unsafe {
        match function.return_type.as_str() {
            "void" => {
                function.cif.call::<()>(function.symbol_ptr, &raw_args);
                Ok(make::nul(()))
            }
            "bool" | "u8" => {
                let result = function.cif.call::<u8>(function.symbol_ptr, &raw_args);
                Ok(make::addr(result as u64))
            }
            "i8" => {
                let result = function.cif.call::<i8>(function.symbol_ptr, &raw_args);
                Ok(make::int(result as i64))
            }
            "u16" => {
                let result = function.cif.call::<u16>(function.symbol_ptr, &raw_args);
                Ok(make::addr(result as u64))
            }
            "i16" => {
                let result = function.cif.call::<i16>(function.symbol_ptr, &raw_args);
                Ok(make::int(result as i64))
            }
            "u32" => {
                let result = function.cif.call::<u32>(function.symbol_ptr, &raw_args);
                Ok(make::addr(result as u64))
            }
            "i32" => {
                let result = function.cif.call::<i32>(function.symbol_ptr, &raw_args);
                Ok(make::int(result as i64))
            }
            "u64" => {
                let result = function.cif.call::<u64>(function.symbol_ptr, &raw_args);
                Ok(make::addr(result as u64))
            }
            "i64" => {
                let result = function.cif.call::<i64>(function.symbol_ptr, &raw_args);
                Ok(make::int(result as i64))
            }
            "f32" => {
                let result = function.cif.call::<f32>(function.symbol_ptr, &raw_args);
                Ok(make::float(result as f64))
            }
            "f64" => {
                let result = function.cif.call::<f64>(function.symbol_ptr, &raw_args);
                Ok(make::float(result))
            }
            "ptr" | "pointer" | "cstr" => {
                let result = function.cif.call::<*mut c_void>(function.symbol_ptr, &raw_args);
                Ok(make::addr(result as u64))
            }
            _ => Err("unsupported return type".to_string()),
        }
    }
}

trait RawValueHolder {
    fn raw_ptr(&self) -> &dyn std::any::Any;
}

struct TypedHolder<T> {
    value: Box<T>,
}

struct CStringHolder {
    _cstring_keepalive: CString,       // keeps memory alive
    ptr: u64,                          // pointer for libffi
}

impl RawValueHolder for CStringHolder {
    fn raw_ptr(&self) -> &dyn std::any::Any {
        &self.ptr as &dyn std::any::Any
    }
}

impl<T: 'static> RawValueHolder for TypedHolder<T> {
    fn raw_ptr(&self) -> &dyn std::any::Any {
        self.value.as_ref()
    }
}

fn box_value(value: RuntimeValue, ty: &str) -> Result<Box<dyn RawValueHolder>, String> {
    unsafe { match (&value.tag, ty) {
        (RT::Addr, "bool" | "u8") => Ok(Box::new(TypedHolder { value: Box::new(value.data.address as u8) })),
        (RT::Int, "i8") => Ok(Box::new(TypedHolder { value: Box::new(value.data.int as i8) })),
        (RT::Addr, "u16") => Ok(Box::new(TypedHolder { value: Box::new(value.data.address as u16) })),
        (RT::Int, "i16") => Ok(Box::new(TypedHolder { value: Box::new(value.data.int as i16) })),
        (RT::Addr, "u32") => Ok(Box::new(TypedHolder { value: Box::new(value.data.address as u32) })),
        (RT::Addr, "i32") => Ok(Box::new(TypedHolder { value: Box::new(value.data.address as i32) })),
        (RT::Int, "i32") => Ok(Box::new(TypedHolder { value: Box::new(value.data.int as i32) })),
        (RT::Int, "u32") => Ok(Box::new(TypedHolder { value: Box::new(value.data.int as u32) })),
        (RT::Addr, "u64" | "ptr" | "pointer") => Ok(Box::new(TypedHolder { value: Box::new(value.data.address as u64) })),
        (RT::Addr, "i64") => Ok(Box::new(TypedHolder { value: Box::new(value.data.int as i64) })),
        (RT::Int, "i64") => Ok(Box::new(TypedHolder { value: Box::new(value.data.int as i64) })),
        (RT::Int, "u64" | "ptr" | "pointer") => Ok(Box::new(TypedHolder { value: Box::new(value.data.address as u64) })),
        (RT::Float, "f32") => Ok(Box::new(TypedHolder { value: Box::new(value.data.float) })),
        (RT::Float, "f64") => Ok(Box::new(TypedHolder { value: Box::new(value.data.float) })),

        // this... exists... Also it is VERY BAD
        // might not even work
        (RT::Str, "ptr" | "pointer" | "cstr") => {
            let s: &str = &value.data.str;
            let c_str = CString::new(s)
                .expect("failed to allocate C string");

            let ptr = c_str.as_ptr() as u64;

            Ok(Box::new(CStringHolder { _cstring_keepalive: c_str, ptr }))
        },

        _ => Err(format!("unsupported argument type: {} for value {:?}", ty, value)),
    } }
}
