//! This module presents an intermediate representation of the VM's instruction
//! set, to be used by the [`compiler`](crate::compiler). This must be
//! translated into it's [`optimized`](crate::vm::inst::optimized) form
//! before it can be executed by the [`VM`](crate::vm).
//! 
//! # Purpose
//! The representation of an optimized instruction can vary, and is dependent
//! on order of insertion.

use crate::inst::RuntimeStaticValue;

#[derive(Debug, Clone, PartialEq)]
// #[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum VMInstruction {
    ////////////////////////////////
    // assignment and declaration //
    ////////////////////////////////
    PushLiteral(RuntimeStaticValue),

    /// Underrated instruction: basically the same thing as
    /// [`VMInstruction::PushLiteral`] but the value is shared.
    /// It's pretty cool! Declared with the `def global` keyword
    /// in the compiler, as it is ideal for global variables.
    PushStatic(RuntimeStaticValue),

    /// Pushes a copy of the given binding
    PushCopy(usize),

    AssignCopy {
        reg: usize,
        new: usize
    },

    AssignLiteral {
        reg: usize,
        new: RuntimeStaticValue
    },

    ///////////////////////
    // regular binaryops //
    ///////////////////////

    /// Adds registers and creates a new value
    AddRegs(usize, usize),

    /// Subtracts registers and creates a new value
    SubRegs(usize, usize),

    /// Multiplies registers and creates a new value
    MulRegs(usize, usize),

    /// Divides registers and creates a new value
    DivRegs(usize, usize),

    /// Preforms modulus registers and creates a new value
    ModRegs(usize, usize),

    /////////////////
    // assign ops  //
    /////////////////
    
    /// Adds registers and assigns to the left register
    AddLeft(usize, usize),

    /// Subtracts registers and assigns to the left register
    SubLeft(usize, usize),

    /// Multiplies registers and assigns to the left register
    MulLeft(usize, usize),

    /// Divides registers and assigns to the left register
    DivLeft(usize, usize),

    /// Preforms modulus registers and assigns to the left register
    ModLeft(usize, usize),

    /////////////////////
    // equality checks //
    /////////////////////
    EqRegs(usize, usize),
    NeqRegs(usize, usize),
    GtRegs(usize, usize),
    LtRegs(usize, usize),

    //////////////////
    // logical ops  //
    //////////////////
    TruthOr(usize, usize),
    TruthAnd(usize, usize),
    TruthNot(usize, usize),

    ////////////////////////////////
    // function and control flow //
    ///////////////////////////////

    /// Calls a function
    Call(usize, Vec<usize>),

    /// Returns the value at the given index
    Ret(usize),

    /// Returns the current scope
    RetScope,

    If {
        cond: usize,
        yes: Vec<VMInstruction>,
        no: Vec<VMInstruction>,
    },

    Loop {
        repeat: Vec<VMInstruction>
    },

    Break,
    Continue,

    ////////////////////////////
    // structs and properties //
    ////////////////////////////

    /// Accesses a property statically
    PropertyStatic {
        lhs: usize,
        rhs: usize,
    },

    ////////////////////////////
    // cleanup and allocation //
    ////////////////////////////
    
    /// Cleans up all of the given indexes off the stack
    /// and removes them from memory
    Purge(Vec<usize>),

    /// Cleans up all of the given indexes off the stack,
    /// but keeps them in memory
    Unbind(Vec<usize>),
}
