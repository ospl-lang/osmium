use std::ffi::c_void;

use ospl_common::inst::optimized::Inst;

#[allow(non_camel_case_types)] pub type cInst = c_void;

#[unsafe(no_mangle)]
pub extern "C" fn OSPL_inst_new() -> *mut cInst {
    let inst = Inst::default();
    return Box::into_raw(Box::new(inst)) as *mut cInst
}

#[unsafe(no_mangle)]
pub extern "C" fn OSPL_inst_destroy(i: *mut cInst) {
    if i.is_null() { return; }
    unsafe {
        drop(Box::from_raw(i));
    }
}

// builder stuff

#[unsafe(no_mangle)]
pub extern "C" fn OSPL_inst_set_opcode(b: *mut cInst, o: u8) {
    if b.is_null() { return; }
    let bp: &mut Inst = unsafe { std::mem::transmute(b) };
    (*bp).opcode = unsafe { std::mem::transmute(o) };
}

#[unsafe(no_mangle)]
pub extern "C" fn OSPL_inst_add_index(b: *mut cInst, i: usize) {
    if b.is_null() { return; }
    let bp: &mut Inst = unsafe { std::mem::transmute(b) };
    bp.indexes.push(i);
}

#[unsafe(no_mangle)]
pub extern "C" fn OSPL_inst_add_child(b: *mut cInst, num: usize, data: *const cInst) {
    if b.is_null() { return; }
    let bp: &mut Inst = unsafe { std::mem::transmute(b) };
    let c = unsafe { std::slice::from_raw_parts(data, num) };
    let c: Vec<Inst> = c.into_iter()
        .map(|e| unsafe { std::mem::transmute::<*const c_void, &Inst>(e) })
        .map(|e| e.clone())
        .collect();

    bp.children.push(c);
}

// getters

#[unsafe(no_mangle)]
pub extern "C" fn OSPL_inst_get_opcode(b: *mut cInst) -> u8 {
    let bp: &mut Inst = unsafe { std::mem::transmute(b) };
    return bp.opcode.clone() as u8
}
