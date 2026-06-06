use std::fmt::Debug;
use serde::Serialize;

// use crate::inst::unoptimized::VMInstruction;
use crate::{ast::frame::RuntimeFrame, inst::optimized::Inst};

pub mod list;
pub mod optimized;
pub mod symbols;

pub mod make {
    use std::mem::ManuallyDrop;
    use crate::{ast::frame::RuntimeFrame, inst::{RV, RuntimeFunction, RuntimeValue, list::List}};

    macro_rules! _make_fn {
        ($f:ident, $rust_type:ty, $tag:ident, $field:ident) => {
            pub fn $f(t:$rust_type) -> $crate::inst::RuntimeValue {
                return $crate::inst::RuntimeValue { tag: $crate::inst::RT::$tag, data: $crate::inst::RV { $field: t }}
            }
        };

        ($f:ident, $rust_type:ty, $tag:ident) => {
            pub fn $f(t:$rust_type) -> $crate::inst::RuntimeValue {
                return $crate::inst::RuntimeValue { tag: $crate::inst::RT::$tag, data: $crate::inst::RV { $f: t }}
            }
        };
    }

    _make_fn!(int, i64, Int);
    _make_fn!(addr, u64, Addr, address);
    _make_fn!(float, f64, Float);
    _make_fn!(bool, bool, Bool);
    _make_fn!(char, char, Char);
    _make_fn!(foreignlib, u32, ForeignLib, foreign);
    _make_fn!(foreignfun, u32, ForeignFn, foreign);
    _make_fn!(nul, (), Nul, nothing);
    _make_fn!(undefined, (), Undefined, nothing);

    pub fn str(s: String) -> RuntimeValue {
        return RuntimeValue { tag: super::RT::Str, data: RV { str: ManuallyDrop::new(s) } }
    }

    pub fn list(s: List) -> RuntimeValue {
        return RuntimeValue { tag: super::RT::List, data: RV { list: ManuallyDrop::new(s) } }
    }

    pub fn func(s: RuntimeFunction) -> RuntimeValue {
        return RuntimeValue { tag: super::RT::Func, data: RV { func: ManuallyDrop::new(s) } }
    }

    pub fn scope(s: RuntimeFrame) -> RuntimeValue {
        return RuntimeValue { tag: super::RT::Scope, data: RV { scope: ManuallyDrop::new(s) } }
    }
}

pub mod assume {
    use crate::{ast::frame::RuntimeFrame, inst::{RuntimeFunction, list::List}};

    #[macro_export] macro_rules! refmut_helper {
        (type, ref, $t:ty) => {
            &$t
        };
        (type, mut, $t:ty) => {
            &mut $t
        };
        (symbol, ref, $e:expr) => {
            &$e
        };
        (symbol, mut, $e:expr) => {
            &mut $e
        };
        (name, mut, $i:ident) => {
            $i_mut
        };
        (name, ref, $i:ident) => {
            $i_ref
        };
    }

    #[macro_export] macro_rules! _assume_fn {
        ($kind:ident, $f:ident, $t:ty, $tag:ident) => {
            pub fn $f(x: refmut_helper!(type, $kind, $crate::inst::RuntimeValue)) -> Option<refmut_helper!(type, $kind, $t)> {
                if x.tag != $crate::inst::RT::$tag {
                    return None
                }

                return Some(unsafe { refmut_helper!(symbol, $kind, x.data.$f) })
            }
        };
        ($kind:ident, $f:ident, $t:ty, $tag:ident, $field:ident) => {
            pub fn $f(x: refmut_helper!(type, $kind, $crate::inst::RuntimeValue)) -> Option<refmut_helper!(type, $kind, $t)> {
                if x.tag != $crate::inst::RT::$tag {
                    return None
                }

                return Some(unsafe { refmut_helper!(symbol, $kind, x.data.$field) })
            }
        };
    }

    _assume_fn!(ref, int, i64, Int);
    _assume_fn!(ref, address, u64, Addr);
    _assume_fn!(ref, float, f64, Float);
    _assume_fn!(ref, bool, bool, Bool);
    _assume_fn!(ref, char, char, Char);
    _assume_fn!(ref, str, String, Str);
    _assume_fn!(ref, list, List, List);
    _assume_fn!(ref, func, RuntimeFunction, Func);
    _assume_fn!(ref, scope, RuntimeFrame, Scope);
    _assume_fn!(ref, foreignlib, u32, ForeignLib, foreign);
    _assume_fn!(ref, foreignfun, u32, ForeignFn, foreign);
    _assume_fn!(ref, nul, (), Nul, nothing);
    _assume_fn!(ref, undefined, (), Undefined, nothing);

}

pub mod assume_mut {
    use crate::{_assume_fn, ast::frame::RuntimeFrame, inst::{RuntimeFunction, list::List}, refmut_helper};
    _assume_fn!(mut, int, i64, Int);
    _assume_fn!(mut, address, u64, Addr);
    _assume_fn!(mut, float, f64, Float);
    _assume_fn!(mut, bool, bool, Bool);
    _assume_fn!(mut, char, char, Char);
    _assume_fn!(mut, str, String, Str);
    _assume_fn!(mut, list, List, List);
    _assume_fn!(mut, func,  RuntimeFunction, Func);
    _assume_fn!(mut, scope, RuntimeFrame, Scope);
    _assume_fn!(mut, foreignlib, u32, ForeignLib, foreign);
    _assume_fn!(mut, foreignfun, u32, ForeignFn, foreign);
    _assume_fn!(mut, nul, (), Nul, nothing);
    _assume_fn!(mut, undefined, (), Undefined, nothing);
}

pub fn make_value(of_type: RT, data: RV) -> RuntimeValue {
    return RuntimeValue { tag: of_type, data }
}

#[derive(Debug, Clone, Copy, PartialEq)]
#[derive(serde::Serialize, serde::Deserialize)]
pub enum RT {
    Int, Addr, Float, Bool, Char, Str,
    List, Func, Scope, ForeignLib, ForeignFn,
    Nul, Undefined,
}

pub struct RuntimeValue {
    pub tag: RT,
    pub data: RV
}

impl PartialEq for RuntimeValue {
    fn eq(&self, other: &Self) -> bool {
        unsafe {
            let a: &[u8] = std::slice::from_raw_parts((&raw const self.data) as *const u8, size_of::<RV>());
            let b: &[u8] = std::slice::from_raw_parts((&raw const other.data) as *const u8, size_of::<RV>());
            a == b
        }
    }
}

impl Debug for RuntimeValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        return write!(f, "{:?}", self.tag)
    }
}

impl Clone for RuntimeValue {
    fn clone(&self) -> Self {
        return Self {
            tag: self.tag,
            data: match self.tag {
                _ => {
                    let mut dst = RV { address: 0 };  // dud value
                    unsafe { std::ptr::copy_nonoverlapping(&raw const self.data, &raw mut dst, 1) };
                    unsafe { std::mem::transmute(dst) }
                }
            }
        }
    }
}

impl Drop for RuntimeValue {
    fn drop(&mut self) {
        unsafe { match self.tag {
            RT::Func => std::ptr::drop_in_place(&mut self.data.func),
            RT::List => std::ptr::drop_in_place(&mut self.data.list),
            RT::Scope => std::ptr::drop_in_place(&mut self.data.scope),
            RT::Str => std::ptr::drop_in_place(&mut self.data.str),
            _ => {}  // no special drop
        } }
    }
}

impl Serialize for RuntimeValue {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: serde::Serializer
    {
        unimplemented!("TODO - IMPL SERDE!")
        // serializer.serialize_u8(self.tag.clone() as u8)
    }
}

pub union RV {
    pub int: i64,
    pub address: u64,
    pub float: f64,
    pub bool: bool,
    pub char: char,
    pub str: std::mem::ManuallyDrop<String>,
    pub list: std::mem::ManuallyDrop<list::List>,
    pub func: std::mem::ManuallyDrop<RuntimeFunction>,
    pub scope: std::mem::ManuallyDrop<RuntimeFrame>,
    pub foreign: u32,
    pub nothing: (),
}

#[derive(Debug, Clone, PartialEq)]
// #[derive(serde::Serialize, serde::Deserialize)]
pub struct RuntimeFunction {
    /// Absolute address
    pub captures: Vec<crate::types::AbsAddress>,
    pub code: Vec<Inst>
}

// pub mod unoptimized;

impl RuntimeValue {
    /// Returns the boolean value (or coerces into one),
    /// or `None` if it is not a boolean.
    pub fn as_bool(&self) -> Option<bool> {
        return match self.tag {
            RT::Bool => Some(unsafe { self.data.bool }),
            RT::Int => Some(unsafe { if self.data.int != 0 {true} else {false} }),
            RT::Addr => Some(unsafe { if self.data.address != 0 {true} else {false} }),
            RT::Float => Some(unsafe { if self.data.float != 0.0 {true} else {false} }),
            _ => None
        }
    }

    /// Returns a reference to [`function::Fn`] if `self` is [`Value::Fn`], and
    /// [`None`] otherwise.
    pub fn as_fn(&self) -> Option<&RuntimeFunction> {
        Some(unsafe { &*self.data.func })
    }

    pub fn as_str(&self) -> Option<&String> {
        Some(unsafe { &*self.data.str })
    }

    /// Unsafely returns the int value (if there is), or garbage data.
    pub unsafe fn assume_int(&self) -> i64 {
        unsafe { self.data.int }
    }

    /// Assumes that this is a foreign element
    pub unsafe fn assume_foreign_element(&self) -> u32 {
        unsafe { self.data.foreign }
    }

    // #[cfg(debug_assertions)]
    // pub unsafe fn assume_int(&self) -> i64 {
    //     return match self {
    //         Self::Int(i) => *i,
    //         other => panic!("{other:?} aint an int!")
    //     }
    // }

    /// Returns the length of the value (if there is one)
    pub fn get_length(&self) -> usize {
        return match self.tag {
            RT::List => unsafe { self.data.list.items.len() },
            RT::Str => unsafe { self.data.str.len() },
            RT::Scope => unsafe { self.data.scope.indexes.len() },
            t => panic!("can't get the len of value of type {t:?}")
        }
    }
}
