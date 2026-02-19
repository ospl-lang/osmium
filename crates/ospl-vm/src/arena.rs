//! This module presents the heart of OSPL, a bump-allocated,
//! garbage-collected, refcounted, continous arena allocator.

use std::fmt::Debug;

use crate::Value;

pub type ArenaIndex = usize;

/// setting this to a power of two makes it very fast for segmented memory in
/// the future! So do that.
pub const MEMMAX: usize = 1024;

pub const FREE_REFCOUNT: usize = usize::MAX;

#[derive(Default, Clone)]
pub struct ArenaItem {
    pub inner: Value,
    pub refcount: usize,
}

impl Debug for ArenaItem {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.refcount == FREE_REFCOUNT {
            return write!(f, "");
        };

        return write!(f, "{}#{:?}", self.refcount, self.inner);
    }
}

impl ArenaItem {
    pub const fn const_default() -> Self {
        return Self {
            refcount: FREE_REFCOUNT,
            inner: Value::Undefined,
        }
    }
}

#[derive(Debug)]
pub struct Arena {
    segment: Box<[ArenaItem; MEMMAX]>,
    freelist: Vec<ArenaIndex>,
}

impl Arena {
    pub fn new() -> Self {
        let mut freelist = Vec::with_capacity(MEMMAX);
        for i in 0..MEMMAX {
            freelist.push(i);
        }

        return Self {
            segment: Box::new([const{ArenaItem::const_default()}; MEMMAX]),
            freelist,
        }
    }

    #[inline(always)]
    pub fn get_item_mut(&mut self, abs: ArenaIndex) -> &mut ArenaItem {
        return &mut self.segment[abs]
    }

    #[inline(always)]
    pub fn reclaim(&mut self, i: ArenaIndex) {
        self.segment[i].inner = Value::Nul;
        self.segment[i].refcount = FREE_REFCOUNT;
        self.freelist.push(i);
    }

    #[inline(always)]
    pub fn push(&mut self, v: Value) -> ArenaIndex {
        if let Some(idx) = self.freelist.pop() {
            self.segment[idx] = ArenaItem {
                refcount: 1,
                inner: v
            };
            return idx;
        } else {
            panic!("we're out of memory")
        }
    }

    #[inline(always)]
    pub fn get(&self, index: ArenaIndex) -> &Value {
        return &self.segment[index].inner
    }
    
    #[inline(always)]
    pub fn get_mut(&mut self, index: ArenaIndex) -> &mut Value {
        return &mut self.segment[index].inner
    }


    #[inline(always)]
    pub unsafe fn raw_get(&self, index: ArenaIndex) -> *const Value {
        return &raw const self.segment[index].inner
    }
    
    #[inline(always)]
    pub unsafe fn raw_get_mut(&mut self, index: ArenaIndex) -> *mut Value {
        return &raw mut self.segment[index].inner
    }
}

