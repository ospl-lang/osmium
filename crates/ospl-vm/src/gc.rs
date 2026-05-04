//! TODO: ADD CYCLE DETECTION.

use crate::{arena::{Arena, ArenaIndex}};

impl Arena {
    /// Decrements a refcount, returning `true` if it's `0`
    /// Does not modify the refcount of objects accessable through it. So it is unsafe
    pub fn dec_refcount(&mut self, abs: ArenaIndex) -> bool {
        let x = self.get_item_mut(abs);
        x.refcount = x.refcount.saturating_sub(1);
        if x.refcount == 0 {
            return true
        }

        return false
    }

    /// Increments a given object's refcount, and all objects reachable by that object
    pub fn inc_refcount(&mut self, abs: ArenaIndex) {
        let x = self.get_item_mut(abs);
        x.refcount += 1;
    }

    pub fn gc_frame_destroyed(&mut self, f: &[usize]) {
        for idx in f {
            if self.dec_refcount(*idx) {
                self.reclaim(*idx);
            }
        }
    }

    pub fn gc_frame_added(&mut self, f: &[usize]) {
        for idx in f {
            self.inc_refcount(*idx);
        }
    }
}