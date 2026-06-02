//! This module presents the heart of OSPL, a bump-allocated,
//! garbage-collected, refcounted, continous arena allocator.

use std::fmt::Debug;

use crate::RuntimeValue;

pub type ArenaIndex = usize;

/// setting this to a power of two makes it very fast for segmented memory in
/// the future! So do that.
pub const MEMMAX: usize = 1024;

// TODO: make this thread-safe!

#[derive(Default, Clone)]
pub struct ArenaItem {
    pub inner: RuntimeValue,
    next_free: Option<usize>,
}

impl ArenaItem {
    pub fn oom() -> Self {
        return Self {
            next_free: None,
            ..Default::default()
        }
    }
}

impl Debug for ArenaItem {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.inner == RuntimeValue::Undefined { 
            return write!(f, "");
        };

        return write!(f, "{:?}", self.inner);
    }
}

impl ArenaItem {
    pub const fn const_default(i: usize) -> Self {
        return Self {
            inner: RuntimeValue::Undefined,
            next_free: Some(i)
        }
    }
}

#[derive(Debug)]
pub struct Arena {
    segment: Box<[ArenaItem; MEMMAX]>,

    /// If this is [`None`], we're out of memory
    freelist_head: Option<usize>
}

impl Arena {
    pub fn new() -> Self {
        let mut v = Vec::with_capacity(MEMMAX);

        for i in 0..MEMMAX {
            v.push(ArenaItem {
                next_free: if i + 1 < MEMMAX {
                    Some(i + 1)
                } else {
                    None
                },
                ..Default::default()
            });
        }

        let segment: Box<[ArenaItem; MEMMAX]> =
            v.into_boxed_slice().try_into().unwrap();

        Self {
            segment,
            freelist_head: Some(0),
        }
    }

    #[inline(always)]
    pub fn get_item_mut(&mut self, abs: ArenaIndex) -> &mut ArenaItem {
        return &mut self.segment[abs]
    }

    #[inline(always)]
    pub fn get_item(&self, abs: ArenaIndex) -> &ArenaItem {
        return &self.segment[abs]
    }

    #[inline(always)]
    pub fn reclaim(&mut self, i: ArenaIndex) {
        let item = &mut self.segment[i];

        // don't change refcount, the caller does that
        item.next_free = self.freelist_head;
        self.freelist_head = Some(i);
    }

    #[inline(always)]
    pub fn push(&mut self, v: RuntimeValue) -> ArenaIndex {
        let head = self.freelist_head.expect("OSPL: out of memory!");
        let item = &mut self.segment[head];
        item.inner = v;

        self.freelist_head = item.next_free;
        return head
    }

    #[inline(always)]
    pub fn get(&self, index: ArenaIndex) -> &RuntimeValue {
        return &self.segment[index].inner
    }
    
    #[inline(always)]
    pub fn get_mut(&mut self, index: ArenaIndex) -> &mut RuntimeValue {
        return &mut self.segment[index].inner
    }

    #[inline(always)]
    pub unsafe fn raw_get(&self, index: ArenaIndex) -> *const RuntimeValue {
        return &raw const self.segment[index].inner
    }
    
    #[inline(always)]
    pub unsafe fn raw_get_mut(&mut self, index: ArenaIndex) -> *mut RuntimeValue {
        return &raw mut self.segment[index].inner
    }
}
