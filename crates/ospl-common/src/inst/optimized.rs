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

use crate::inst::{RuntimeValue, symbols::DebugSymbolTable};

#[derive(Debug, Clone, PartialEq, Eq, Default)]
#[derive(serde::Serialize, serde::Deserialize)]
pub enum Opc {
    /// Does nothing
    #[default]
    NullOp = 0,

    PushLiteral = 1,

    /// Pushes an array to the stack.
    /// 
    /// - **Indexes:** the members of the array (the refcounts will be
    ///                incremented)
    PushArray = 2,

    /// Pushes a function, except the captures are specified as
    /// relative addresses, and are converted to 
    /// 
    /// This is the correct way to push a function, it is not
    /// recommended (and actually a violation of the spec) to
    /// use PushLiteral to create a function, dear compilers.
    /// 
    /// - **Indexes:** the captures of the function
    /// - **Child #0:** the code of the function
    PushFunction = 3,

    AssignRef = 4,
    AssignLiteral = 5,

    /** Binary `+` */ Add = 6,
    /** Binary `-` */ Sub = 7,
    /** Binary `*` */ Mul = 8,
    /** Binary `/` */ Div = 9,
    /** Binary `%` */ Mod = 10,

    /** Assign `+=` */ Addl = 11,
    /** Assign `-=` */ Subl = 12,
    /** Assign `*=` */ Mull = 13,
    /** Assign `/=` */ Divl = 14,
    /** Assign `%=` */ Modl = 15,

    /** Unary `--`
     * 
     * When used on a list, the value is popped off instead
    */ Decrement = 16,

    /** Unary `++` */ Increment = 17,

    Eq = 18,
    Neq = 19,
    Gt = 20,
    Lt = 21,
    Gte = 22,
    Lte = 23,

    Neg = 24,

    Lnot = 25,
    Lor = 26,
    Land = 27,

    /// The `?` (find in) operator.
    /// 
    /// - **Index #0:** the index of the thing to find in
    /// - **Index #1:** the index of the thing to find in
    QuestionMark = 28,

    Call = 29,
    Ret = 30,
    RetScope = 31,

    If = 32,
    Loop = 33,

    Break = 34,
    Continue = 35,

    /// Gets the length of an array or string.
    /// 
    /// - **Index #0:** the array to get the length of
    /// - **Return:** the length as an Int
    GetLength = 36,

    /// Pops a value at off a sequence
    /// 
    /// - **Index #0:** the array/string to pop off of
    /// - **Index #1:** a ref to an Int containing the value to index
    Pop = 37,

    /** Get a property */ Property = 38,
    /** Index into an array */ Index = 39,
    /** Slice into an array */ Slice = 40,

    /* ********************************************************************* */
    /*           FFI STUFF                                                   */
    /* ********************************************************************* */

    /// Loads a foreign library from a file
    /// 
    /// - **Index #0:** a string value indicating the library to load
    /// - **Return:** a [`RuntimeValue::ForeignLib`]
    FFILoadLib = 41,

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
    FFILoadFn = 42,

    FFICall = 43,

    /// Converts a (primitive) value to a type and pushes it onto the stack.
    /// 
    /// - **Index #0:** the value to convert
    /// - **Index #1:** the type primitive type number to convert to (see
    /// [`crate::ast::Type::to_primitive_type_id`])
    Cast = 44,

    /// Copies the left argument
    Copy = 45,
}

#[derive(Debug, Clone, PartialEq, Default)]
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
    pub children: Vec<Vec<Inst>>,

    pub debug_symbol: Option<usize>,
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
                debug_symbol: None,
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

    pub fn symbol(mut self, table: &mut DebugSymbolTable, s: String) -> Self {
        let x = table.get_or_add(s);
        self.inner.debug_symbol = Some(x);

        return self
    }

    pub fn build(self) -> Inst {
        return self.inner
    }
}
