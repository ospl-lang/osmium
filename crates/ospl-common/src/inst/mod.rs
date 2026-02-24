use crate::inst::unoptimized::VMInstruction;

/// A value in it's static runtime form
#[derive(Debug, Clone, PartialEq)]
pub enum RuntimeStaticValue {
    Int(i64),
    Float(f64),
    Function(RuntimeFunctionData),
    Structure(RuntimeStructureData),
    Nul,
}

#[derive(Debug, Clone, PartialEq)]
pub struct RuntimeFunctionData {
    pub code: Vec<VMInstruction>    
}

#[derive(Debug, Clone, PartialEq)]
pub struct RuntimeStructureData {
    pub items: Vec<usize>
}

pub mod unoptimized;
