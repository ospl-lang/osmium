use std::ffi::{CStr, c_char};
use ospl_common::inst::{RuntimeValue, make};

macro_rules! _make_fn_prim {
    ($f:ty, $x:ident, $p:ident) => {
        #[unsafe(no_mangle)]
        pub extern "C" fn $p(v: $f) -> *mut ospl_common::inst::RuntimeValue {
            return Box::into_raw(Box::new(ospl_common::inst::make::$x(v)));
        }
    };
}

_make_fn_prim!(i64, int, OSPL_value_new_int);
_make_fn_prim!(u64, addr, OSPL_value_new_addr);
_make_fn_prim!(f64, float, OSPL_value_new_float);
_make_fn_prim!(bool, bool, OSPL_value_new_bool);

#[unsafe(no_mangle)]
pub extern "C" fn OSPL_value_new_str(v: *const c_char) -> *mut RuntimeValue {
    let str = unsafe {
        let cstr = CStr::from_ptr(v);
        let str = cstr.to_str().unwrap().to_string();
        str
    };
    return Box::into_raw(Box::new(make::str(str)));
}

#[unsafe(no_mangle)]
pub extern "C" fn OSPL_value_new_undefined() -> *mut RuntimeValue {
    return Box::into_raw(Box::new(make::undefined(())));
}

#[unsafe(no_mangle)]
pub extern "C" fn OSPL_value_new_nul() -> *mut RuntimeValue {
    return Box::into_raw(Box::new(make::nul(())));
}

#[unsafe(no_mangle)]
pub extern "C" fn OSPL_value_new_char(c: u32) -> *mut RuntimeValue {
    let Some(x) = char::from_u32(c)
    else { return std::ptr::null_mut(); };

    return Box::into_raw(Box::new(make::char(x)));
}
