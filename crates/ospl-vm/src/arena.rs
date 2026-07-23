use std::fmt::Debug;

pub type ArenaIndex = usize;

pub const MEMMAX: usize = 1024;

#[derive(Clone)]
pub struct ArenaItem<T> {
    pub inner: Option<T>,
    pub(crate) next_free: usize,
}

impl<T> ArenaItem<T> {
    pub fn oom() -> Self 
    where
        T: Default,
    {
        Self {
            next_free: usize::MAX,
            inner: Some(T::default()),
        }
    }
}

impl<T: Debug> Debug for ArenaItem<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self.inner)
    }
}

#[derive(Debug)]
pub struct Arena<T> {
    segment: Box<[ArenaItem<T>; MEMMAX]>,

    /// If this is [`usize::MAX`], we're out of memory
    freelist_head: usize,
}

impl<T: Debug> Arena<T> {
    pub fn new(default: T) -> Self
    where
        T: Clone,
    {
        let mut v = Vec::with_capacity(MEMMAX);

        for i in 0..MEMMAX {
            v.push(ArenaItem {
                next_free: if i + 1 < MEMMAX {
                    i + 1
                } else {
                    usize::MAX
                },
                inner: Some(default.clone()),
            });
        }

        let segment: Box<[ArenaItem<T>; MEMMAX]> =
            v.into_boxed_slice().try_into().unwrap();

        Self {
            segment,
            freelist_head: 0,
        }
    }

    #[inline(always)]
    pub fn get_item_mut(&mut self, abs: ArenaIndex) -> &mut ArenaItem<T> {
        &mut self.segment[abs]
    }

    #[inline(always)]
    pub fn get_item(&self, abs: ArenaIndex) -> &ArenaItem<T> {
        &self.segment[abs]
    }

    #[inline(always)]
    pub fn reclaim(&mut self, i: ArenaIndex) {
        let item = &mut self.segment[i];

        if item.inner.is_none() {
            return;
        }

        item.next_free = self.freelist_head;
        item.inner = None;
        self.freelist_head = i;
    }

    #[inline(always)]
    pub fn push(&mut self, v: T) -> ArenaIndex {
        let head = self.freelist_head;

        if head == usize::MAX {
            panic!("OSPL: OOM");
        }

        let item = &mut self.segment[head];
        item.inner = Some(v);

        self.freelist_head = item.next_free;

        head
    }

    #[inline(always)]
    pub fn get(&self, index: ArenaIndex) -> &T {
        self.segment[index]
            .inner
            .as_ref()
            .unwrap()
    }

    #[inline(always)]
    pub fn get_mut(&mut self, index: ArenaIndex) -> &mut T {
        self.segment[index]
            .inner
            .as_mut()
            .unwrap()
    }

    #[inline(always)]
    pub unsafe fn raw_get(&self, index: ArenaIndex) -> *const T {
        &raw const *self.segment[index].inner.as_ref().unwrap()
    }

    #[inline(always)]
    pub unsafe fn raw_get_mut(&mut self, index: ArenaIndex) -> *mut T {
        &raw mut *self.segment[index].inner.as_mut().unwrap()
    }
}