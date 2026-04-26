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

use crate::inst::RuntimeStaticValue;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Opc {
    /// Does nothing
    NullOp,

    PushLiteral,

    PushCopy,
    AssignCopy,
    AssignLiteral,

    Add,
    Sub,
    Mul,
    Div,
    Mod,

    Addl,
    Subl,
    Mull,
    Divl,
    Modl,

    Eq,
    Neq,
    Gt,
    Lt,
    Gte,
    Lte,

    Lnot,
    Lor,
    Land,

    Call,
    Ret,
    RetScope,

    If,
    Loop,

    Break,
    Continue,

    PropertyStatic,

    Purge,
    Unbind,
}

#[derive(Debug, Clone, PartialEq)]
// #[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Inst {
    pub opcode: Opc,

    // can contain various types of opcodes.
    // although most will contain operand lists

    /// Indexes for this operation
    pub indexes: Vec<usize>,

    /// Immediate value for this operation, if any
    pub immediate: Option<RuntimeStaticValue>,

    /// Children (child instructions) for this, if any.
    /// 
    /// Mostly used for if statements and loops
    pub children: Vec<Vec<Inst>>
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

    pub fn child(mut self, child: Vec<Inst>) -> Self {
        self.inner.children.push(child);

        return self
    }

    pub fn value(mut self, x: RuntimeStaticValue) -> Self {
        self.inner.immediate = Some(x);

        return self
    }

    pub fn build(self) -> Inst {
        return self.inner
    }
}
