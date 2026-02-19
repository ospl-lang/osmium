use crate::base::{Scope, Type};
use ::ospl_common::inst::VMInstruction;

pub mod base;
mod impls;

pub mod tests;

#[derive(Default)]
pub struct Compiler {
    pub scope: Scope,
    pub code: Vec<VMInstruction>
}

pub struct BinaryOpEval {
    pub left: usize,
    pub right: usize,
    pub result: usize,
}

pub struct Eval {
    pub ty: Type,
    pub index: usize,
}
