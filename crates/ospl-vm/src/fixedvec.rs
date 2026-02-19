//! This module presents a fixed-sized vector, designed for storing
//! small, `Copy` types, and fast cloning.
//! 
//! This should be a drop-in replacement for [`Vec<T>`]

use std::{fmt::Debug, mem::MaybeUninit, ops::{Index, IndexMut}};
use std::fmt::Write;

pub struct FixedVec<T: Copy, const S: usize> {
    inner: [MaybeUninit<T>; S],
    len: usize
}

impl<T: Copy, const S: usize> Default for FixedVec<T, S> {
    fn default() -> Self {
        return Self::new();
    }
}

impl<T: Copy + Debug, const S: usize> Debug for FixedVec<T, S> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut out = String::new();
        for index in 0..self.len {
            let obj = &self[index];
            write!(&mut out, "{:?}", obj)?;
        }

        return f.write_str(&out)
    }
}

impl<T: Copy, const S: usize> FixedVec<T, S> {
    pub fn push(&mut self, x: T) {
        self.inner[self.len] = MaybeUninit::new(x);
        self.len += 1;
    }

    pub fn new() -> Self {
        let indexes: [MaybeUninit<T>; S] = unsafe { MaybeUninit::uninit().assume_init() };

        return FixedVec {
            inner: indexes,
            len: 0
        }
    }

    pub unsafe fn inner_ref(&self) -> &[MaybeUninit<T>; S] {
        return &self.inner
    }

    pub unsafe fn inner_mut(&mut self) -> &mut [MaybeUninit<T>; S] {
        return &mut self.inner
    }

    pub fn as_slice(&self) -> &[T] {
        // SAFETY: we only read the first `len` entries, which have been initialized
        unsafe {
            &*(&self.inner[..self.len] as *const [MaybeUninit<T>] as *const [T])
        }
    }

    pub fn extend(&mut self, from: FixedVec<T, S>) {
        self.inner[self.len..self.len+from.len].copy_from_slice(&from.inner[..from.len]);
        self.len += from.len;
    }

    pub fn extend_from_slice(&mut self, from: &[T]) {
        for thing in from.iter() {
            self.inner[self.len] = MaybeUninit::new(*thing);
            self.len += 1;
        }
    }

    pub fn len(&self) -> usize {
        return self.len;
    }
    
    pub unsafe fn iter_inner(&self) -> impl Iterator<Item = &T> {
        return self.inner
            .iter()
            .map(|x| unsafe{x.assume_init_ref()});
    }
}

impl<T: Copy, const S: usize> Clone for FixedVec<T, S> {
    fn clone(&self) -> Self {
        let mut indexes: [MaybeUninit<T>; S] = unsafe { MaybeUninit::uninit().assume_init() };

        indexes[..self.len].copy_from_slice(&self.inner[..self.len]);

        return FixedVec {
            inner: indexes,
            len: self.len
        }
    }
}

impl<T: Copy, const S: usize> Index<usize> for FixedVec<T, S> {
    type Output = T;
    fn index(&self, index: usize) -> &Self::Output {
        return unsafe {self.inner[index].assume_init_ref()}
    }
}

impl<T: Copy, const S: usize> IndexMut<usize> for FixedVec<T, S> {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        return unsafe {self.inner[index].assume_init_mut()}
    }
}
