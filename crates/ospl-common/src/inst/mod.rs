// use crate::inst::unoptimized::VMInstruction;

use crate::{ast::frame::RuntimeFrame, inst::optimized::Inst};

/// A value in it's static runtime form
#[derive(Default, Debug, Clone, PartialEq)]
pub enum RuntimeValue {
    Int(i64),
    Float(f64),
    Bool(bool),
    Function(RuntimeFunction),
    Scope(Box<RuntimeFrame>),

    Nul,

    #[default]
    Undefined,
}

#[derive(Debug, Clone, PartialEq)]
pub struct RuntimeFunction {
    pub lexical_indexes: Vec<usize>,
    pub code: Vec<Inst>    
}

// pub mod unoptimized;
pub mod optimized;

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

    // pub fn exactly_equal(&self, other: &Self) -> bool {
    //     return match (self, other) {
    //         (Self::Nul, Self::Nul) => true,
    //         (Self::Undefined, Self::Undefined) => true,
    //         (Self::Int(x), Self::Int(y)) => x == y,
    //         (Self::Float(x), Self::Float(y)) => x != y,
    //         _ => false
    //     };
    // }

    /// Returns a reference to [`function::Fn`] if `self` is [`Value::Fn`], and
    /// [`None`] otherwise.
    pub fn as_fn(&self) -> Option<&RuntimeFunction> {
        return match self {
            Self::Function(x) => Some(&x),
            _ => None
        }
    }
}
