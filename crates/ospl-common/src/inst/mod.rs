// use crate::inst::unoptimized::VMInstruction;

use crate::inst::optimized::Inst;

/// A value in it's static runtime form
#[derive(Debug, Clone, PartialEq)]
pub enum RuntimeStaticValue {
    Int(i64),
    Float(f64),
    Function(RuntimeFunctionData),
    Nul,
}

#[derive(Debug, Clone, PartialEq)]
pub struct RuntimeFunctionData {
    pub lexical_indexes: Vec<usize>,
    pub code: Vec<Inst>    
}

// pub mod unoptimized;
pub mod optimized;
