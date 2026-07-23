use ospl_common::inst::RT;
use crate::{VM, arena::MEMMAX};

/// How many runs of the GC does it take to max out a segment.
pub const RUNS_PER_MEMMAX: usize = (MEMMAX + 63) / 64;

#[derive(Debug, Default, Clone)]
pub struct BitSet {
    pub bits: Vec<u64>,
}

impl BitSet {
    pub fn new(size: usize) -> Self {
        let words = (size + 63) / 64;

        Self {
            bits: vec![0; words],
        }
    }

    pub fn resize(&mut self, size: usize) {
        let words = (size + 63) / 64;
        self.bits.resize(words, 0);
    }

    #[inline(always)]
    pub fn set(&mut self, i: usize) {
        self.bits[i >> 6] |= 1 << (i & 63);
    }

    #[inline(always)]
    pub fn contains(&self, i: usize) -> bool {
        (self.bits[i >> 6] & (1 << (i & 63))) != 0
    }

    #[inline(always)]
    pub fn clear(&mut self) {
        self.bits.fill(0);
    }
}

#[derive(Default, Debug)]
pub struct GC {
    pub cursor: usize,
}

impl VM {
    #[inline(always)]
    fn trace_value(&self, index: usize, out: &mut BitSet) {
        // had to add this check to prevent an OOB crash
        // if index >= GC_BYTES { return; }

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

    pub fn mark(&mut self) -> BitSet {
        let size = self.arena.segment_count() * MEMMAX;
        let mut marked = BitSet::new(size);

        for root in self.stack.frames.iter() {
            for index in self.stack.data[root.base..root.base + root.size].iter() {
                self.trace_value(*index, &mut marked);
            }
        }

        marked
    }

    #[inline(always)]
    fn sweep_word(&mut self, marked: &BitSet, word_index: usize) {
        let Some(dead) = marked.bits.get(word_index)
        else { return; };

        let mut dead = !*dead;

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
        let marked = self.mark();
        let mut out = false;
        for _ in 0..need {
            self.sweep_word(&marked, self.gc.cursor);

            let words = marked.bits.len();
            if self.gc.cursor >= words {
                self.reset_gc();
                out = true;
            }
        }

        if self.arena.segment_empty(self.arena.segment_count() - 1) {
            self.arena.pop_segment();
        }

        return out
    }
}
