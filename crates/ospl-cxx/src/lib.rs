use std::ffi::c_void;

pub const HEADER_FILE: &str = include_str!("ospl.h");

use ospl_common::inst::optimized::Inst;
use ospl_vm::VM;

use crate::inst::cInst;

pub mod make;
pub mod inst;

#[allow(non_camel_case_types)] pub type cVM = c_void;

#[unsafe(no_mangle)]
pub extern "C" fn OSPL_vm_create() -> *mut cVM {
    return Box::into_raw(Box::new(ospl_vm::VM::new())) as *mut cVM
}

#[unsafe(no_mangle)]
pub extern "C" fn OSPL_vm_protected_run(
    f: *mut extern "C" fn(),
    vm: *mut cVM,
    i: *mut cInst
)
{
    let e = std::panic::catch_unwind(|| {
        let vv: &mut VM = unsafe { std::mem::transmute(vm) };
        let bp: &mut Inst = unsafe { std::mem::transmute(i) };
        vv.run_one(bp);
    });

    match e {
        Err(_) => unsafe { (*f)() },
        Ok(()) => {}
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn OSPL_vm_destroy(vm: *mut cVM) {
    if vm.is_null() { return; }
    unsafe {
        drop(Box::from_raw(vm));
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn OSPL_vm_run(vm: *mut cVM, i: *mut cInst) {
    let vv: &mut VM = unsafe { std::mem::transmute(vm) };
    let bp: &mut Inst = unsafe { std::mem::transmute(i) };
    vv.run_one(bp);
}
