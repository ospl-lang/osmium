//! This module presents an optimized version of the bytecode, replacing
//! `VMInstruction`.
//! 
//! # Purpose
//! It is implemented like this because we don't want the compiler to generate
//! many tag checks, because that would decode-bottleneck us. So we want
//! to generate a big jump table instead.
//! 
//! This is tehcnically a bit anti-Rust, but we care about speed, not
//! completely idiomatic code.
//! 
//! # Items
//! This module supplies a bytecode representation ([`Opc`]
//! and [`Inst`]). As well as a builder API for this new
//! representation ([`InstBuilder`])

use std::fmt::Display;

use crate::inst::RuntimeValue;

#[derive(Debug, Clone, PartialEq, Eq)]
#[derive(serde::Serialize, serde::Deserialize)]
pub enum Opc {
    /// Does nothing
    NullOp,

    PushLiteral,

    /// Pushes a frame onto the stack, but not in the way a function would.
    /// This is intended for generic compiler namespaces such as modules.
    /// 
    /// - **Indexes:** the members of the scope relative to the current one
    ///                 (the refcounts will be incremented)
    PushFrame,

    /// Pushes an array to the stack.
    /// 
    /// - **Indexes:** the members of the array (the refcounts will be
    ///                incremented)
    PushArray,

    /// Pushes a function, except the captures are specified as
    /// relative addresses, and are converted to 
    /// 
    /// This is the correct way to push a function, it is not
    /// recommended (and actually a violation of the spec) to
    /// use PushLiteral to create a function, dear compilers.
    /// 
    /// - **Indexes:** the captures of the function
    /// - **Child #0:** the code of the function
    PushFunction,

    /// Pushes a function and then immediately invokes it.
    /// This is the optimized path for IIFEs, good compilers should use it.
    /// 
    /// - **Indexes:** the args
    /// - **Child 0:** the body
    /// - **Push 0:** the function
    /// - **Push 1:** the return value
    IIFE,

    AssignCopy,
    AssignRef,
    AssignLiteral,

    /** Binary `+` */ Add,
    /** Binary `-` */ Sub,
    /** Binary `*` */ Mul,
    /** Binary `/` */ Div,
    /** Binary `%` */ Mod,

    /** Assign `+=` */ Addl,
    /** Assign `-=` */ Subl,
    /** Assign `*=` */ Mull,
    /** Assign `/=` */ Divl,
    /** Assign `%=` */ Modl,

    /** Unary `--`
     * 
     * When used on a list, the value is popped off instead
    */ Decrement,

    /** Unary `++` */ Increment,

    Eq,
    Neq,
    Gt,
    Lt,
    Gte,
    Lte,

    Neg,

    Lnot,
    Lor,
    Land,

    /// The `?` (find in) operator.
    /// 
    /// - **Index #0:** the index of the thing to find in
    /// - **Index #1:** the index of the thing to find in
    QuestionMark,

    Call,
    Ret,
    RetScope,

    If,
    Loop,

    Break,
    Continue,

    /// Gets the length of an array or string.
    /// 
    /// - **Index #0:** the array to get the length of
    /// - **Return:** the length as an Int
    GetLength,

    /// Pops a value at off a sequence
    /// 
    /// - **Index #0:** the array/string to pop off of
    /// - **Index #1:** a ref to an Int containing the value to index
    Pop,

    /** Get a property */ Property,
    /** Index into an array */ Index,
    /** Slice into an array */ Slice,

    /* ********************************************************************* */
    /*           FFI STUFF                                                   */
    /* ********************************************************************* */

    /// Loads a foreign library from a file
    /// 
    /// - **Index #0:** a string value indicating the library to load
    /// - **Return:** a [`RuntimeValue::ForeignLib`]
    FFILoadLib,

    /// Loads a foreign function from a library
    /// 
    /// - **Index #0:** a string value indicating which library to load from
    /// - **Index #1:** a string value indicating which functon to obtain
    /// - **Index #2:** a number (not a reference to a number, just a number
    ///                 as the index) that represents the return type.
    /// 
    /// - **Remaining indexes:** a number that represents each argument type.
    /// 
    /// - **Return:** a [`RuntimeValue::ForeignFn`]
    /// 
    /// Type enum mapping:
    /// - 0: u8
    /// - 1: i8
    /// - 2: u16
    /// - 3: i16
    /// - 5: u32
    /// - 6: i32
    /// - 7: u64
    /// - 8: i64
    /// - 9: float
    /// - 10: double
    /// - 11: ptr
    FFILoadFn,

    FFICall,

    /// Converts a (primitive) value to a type and pushes it onto the stack.
    /// 
    /// - **Index #0:** the value to convert
    /// - **Index #1:** the type primitive type number to convert to (see
    /// [`crate::ast::Type::to_primitive_type_id`])
    Cast,
}

#[derive(Debug, Clone, PartialEq)]
#[derive(serde::Serialize, serde::Deserialize)]
pub struct Inst {
    pub opcode: Opc,

    // can contain various types of opcodes.
    // although most will contain operand lists

    /// Indexes for this operation
    pub indexes: Vec<usize>,

    /// Immediate value for this operation, if any
    pub immediate: Option<RuntimeValue>,

    /// Children (child instructions) for this, if any.
    /// 
    /// Mostly used for if statements and loops
    pub children: Vec<Vec<Inst>>
}

impl Display for Inst {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        return write!(f, "{:?}\t{:?}\t{:?}\t[{:#?}]", self.opcode, self.immediate, self.indexes, self.children);
    }
}

impl Inst {
    #[cfg(not(debug_assertions))]
    pub fn get_index(&self, x: usize) -> usize {
        return unsafe { *self.indexes.get_unchecked(x) }
    }

    #[cfg(debug_assertions)]
    pub fn get_index(&self, x: usize) -> usize {
        return self.indexes[x]
    }
}

pub struct InstBuilder {
    inner: Inst
}

impl InstBuilder {
    pub fn new() -> Self {
        return Self {
            inner: Inst {
                opcode: Opc::NullOp,
                indexes: Vec::new(),
                immediate: None,
                children: Vec::new(),
            }
        }
    }

    pub fn opcode(mut self, opc: Opc) -> Self {
        self.inner.opcode = opc;
        return self
    }

    pub fn index(mut self, index: usize) -> Self {
        self.inner.indexes.push(index);
        return self
    }

    pub fn indexes(mut self, indexes: &[usize]) -> Self {
        self.inner.indexes.extend_from_slice(indexes);
        return self
    }

    pub fn child(mut self, child: Vec<Inst>) -> Self {
        self.inner.children.push(child);

        return self
    }

    pub fn value(mut self, x: RuntimeValue) -> Self {
        self.inner.immediate = Some(x);

        return self
    }

    pub fn build(self) -> Inst {
        return self.inner
    }
}
