//! TODO: ADD CYCLE DETECTION.

use ospl_common::inst::RuntimeValue;

use crate::{VM, arena::MEMMAX};

pub const BITS: usize = (MEMMAX + 63) / 64;

pub struct BitSet<const N: usize> {
    bits: [u64; N],
}

impl<const N: usize> BitSet<N> {
    pub fn new() -> Self {
        Self { bits: [0; N] }
    }

    pub fn set(&mut self, i: usize) {
        self.bits[i / 64] |= 1 << (i % 64);
    }

    pub fn contains(&self, i: usize) -> bool {
        (self.bits[i / 64] & (1 << (i % 64))) != 0
    }
}

impl VM {
    fn trace_value(&self, index: usize, out: &mut BitSet<BITS>) {
        out.set(index);
        let value = self.arena.get(index);
        match value {
            RuntimeValue::Function(f) => for index in &f.captures {
                self.trace_value(*index, out);
            }
            RuntimeValue::List(l) => for index in &l.items {
                self.trace_value(*index, out);
            }
            RuntimeValue::Scope(s) => for index in &s.indexes {
                self.trace_value(*index, out);
            }
            _ => {}
        }
    }

    pub fn gc(&mut self) {
        let mut marked: BitSet<BITS> = BitSet::new();
        for root in self.stack.iter() {
            for index in root.indexes.iter() {
                self.trace_value(*index, &mut marked);
            }
        }

        // O(n + k)
        for i in 0..MEMMAX {
            if !marked.contains(i) {
                self.arena.reclaim(i);
            }
        }
    }
}
