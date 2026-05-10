// use crate::inst::unoptimized::VMInstruction;
use crate::{ast::frame::RuntimeFrame, inst::optimized::Inst};

pub mod list;
pub mod optimized;

#[derive(Default, Debug, Clone, PartialEq)]
#[derive(serde::Serialize, serde::Deserialize)]
pub enum RuntimeValue {
    Int(i64),
    Address(u64),
    Float(f64),
    Bool(bool),
    Char(char),
    Str(String),
    List(list::List),
    Function(RuntimeFunction),
    Scope(RuntimeFrame),

    ForeignLib(u32),
    ForeignFn(u32),

    Nul,

    #[default]
    Undefined,
}

#[derive(Debug, Clone, PartialEq)]
#[derive(serde::Serialize, serde::Deserialize)]
pub struct RuntimeFunction {
    /// Absolute address
    pub lexical_indexes: Vec<crate::types::AbsAddress>,
    pub code: Vec<Inst>
}

// pub mod unoptimized;

impl RuntimeValue {
    /// Returns the boolean value (or coerces into one),
    /// or `None` if it is not a boolean.
    pub fn as_bool(&self) -> Option<bool> {
        return match self {
            Self::Bool(x) => Some(*x),
            Self::Int(x) => Some(if *x != 0 {true} else {false}),
            _ => None
        }
    }

    /// Returns a reference to [`function::Fn`] if `self` is [`Value::Fn`], and
    /// [`None`] otherwise.
    pub fn as_fn(&self) -> Option<&RuntimeFunction> {
        return match self {
            Self::Function(x) => Some(&x),
            _ => None
        }
    }

    /// Unsafely returns the int value (if there is), or garbage data.
    #[cfg(not(debug_assertions))]
    pub unsafe fn assume_int(&self) -> i64 {
        return match self {
            Self::Int(i) => *i,
            _ => unsafe { std::hint::unreachable_unchecked() }
        }
    }

    #[cfg(debug_assertions)]
    pub unsafe fn assume_int(&self) -> i64 {
        return match self {
            Self::Int(i) => *i,
            other => panic!("{other:?} aint an int!")
        }
    }

    /// Returns the length of the value (if there is one)
    pub fn get_length(&self) -> usize {
        return match self {
            Self::List(i) => i.items.len(),
            Self::Str(s) => s.len(),
            t => panic!("can't get the len of value of type {t:?}")
        }
    }
}
