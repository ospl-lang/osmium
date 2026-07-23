use ospl_common::inst::RT;

use crate::{VM, arena::MEMMAX};

pub const GC_BYTES: usize = (MEMMAX + 63) / 64;

#[derive(Debug)]
pub struct BitSet<const N: usize> {
    bits: [u64; N],
}

impl<const N: usize> Default for BitSet<N> {
    fn default() -> Self {
        return Self {
            bits: [0u64; N]
        }
    }
}

impl<const N: usize> BitSet<N> {
    #[inline(always)]
    pub fn new() -> Self {
        Self { bits: [0; N] }
    }

    #[inline(always)]
    pub fn set(&mut self, i: usize) {
        self.bits[i >> 6] |= 1 << (i & 63);
    }

    #[inline(always)]
    pub fn contains(&self, i: usize) -> bool {
        (self.bits[i >> 6] & (1 << (i & 63))) != 0
    }
}

#[derive(Default, Debug)]
pub struct GC {
    pub cursor: usize,
    pub marked: BitSet<GC_BYTES>,
}

impl VM {
    #[inline(always)]
    fn trace_value(&self, index: usize, out: &mut BitSet<GC_BYTES>) {
        if out.contains(index) {
            return;
        }

        out.set(index);

        let value = self.arena.get(index);

        match value.tag {
            RT::Func => {
                for &capture in unsafe { &value.data.func.captures } {
                    self.trace_value(capture, out);
                }
            }
            RT::List => {
                for &item in unsafe { &value.data.list.items } {
                    self.trace_value(item, out);
                }
            }
            RT::Scope => {
                for &item in unsafe { &value.data.scope.indexes } {
                    self.trace_value(item, out);
                }
            }
            _ => {}
        }
    }

    #[inline(always)]
    pub fn mark(&mut self) -> BitSet<GC_BYTES> {
        let mut marked: BitSet<GC_BYTES> = BitSet::new();
        for root in self.stack.frames.iter() {
            for index in self.stack.data[root.base..root.base + root.size].iter() {
                self.trace_value(*index, &mut marked);
            }
        }

        return marked
    }

    #[inline(always)]
    fn sweep_word(&mut self, word_index: usize) {
        self.gc.marked = self.mark();
        let mut dead = !self.gc.marked.bits[word_index];

        while dead != 0 {
            let bit = dead.trailing_zeros() as usize;
            let index = word_index * 64 + bit;

            self.arena.reclaim(index);

            dead &= dead - 1;
        }

        self.gc.cursor += 1;
    }

    #[inline(always)]
    pub fn reset_gc(&mut self) {
        self.gc.cursor = 0;
    }

    #[inline(never)]
    pub fn gc_step(&mut self, need: usize) -> bool {
        for _ in 0..need {
            if self.gc.cursor >= GC_BYTES {
                self.reset_gc();
                return true
            }
            self.sweep_word(self.gc.cursor);
        }

        return false
    }
}
