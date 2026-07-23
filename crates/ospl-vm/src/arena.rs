use std::fmt::Debug;

pub type ArenaIndex = usize;

pub const MEMMAX: usize = 1024;

#[derive(Clone, PartialEq)]
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
pub struct Arena<T: Clone> {
    segments: Vec<Box<[ArenaItem<T>; MEMMAX]>>,

    /// If this is [`usize::MAX`], we're out of memory
    freelist_head: usize,

    default: Option<T>
}

impl<T: Debug + Clone> Arena<T> {
    #[inline(never)]
    pub fn new_sealed_segment(default: Option<T>, base: usize) -> Box<[ArenaItem<T>; MEMMAX]> {
        let mut v = Vec::with_capacity(MEMMAX);

        for i in 0..MEMMAX {
            let out_i = base + i;
            v.push(ArenaItem {
                next_free: if i + 1 < MEMMAX {
                    out_i + 1
                } else {
                    usize::MAX
                },
                inner: default.clone(),
            });
        }

        let segment: Box<[ArenaItem<T>; MEMMAX]> =
            v.into_boxed_slice().try_into().unwrap();

        return segment
    }

    pub fn segment_count(&self) -> usize {
        self.segments.len()
    }

    pub fn get_segments(&self) -> &[Box<[ArenaItem<T>; MEMMAX]>] {
        return self.segments.as_slice()
    }

    pub fn pop_segment(&mut self) -> bool {
        if self.segments.len() == 1 {
            // can't remove the last segment
            return false;
        }

        let old_last = self.segments.len() - 1;
        let old_tail = old_last * MEMMAX - 1;

        // Disconnect previous segment from the segment we're removing.
        self.segments[old_tail / MEMMAX][old_tail % MEMMAX].next_free = usize::MAX;

        self.segments.pop();

        // If the freelist was pointing into the removed segment,
        // restore it to the end of the remaining heap.
        if self.freelist_head >= old_last * MEMMAX {
            self.freelist_head = old_tail;
        }

        return true
    }

    pub fn segment_empty(&self, segment: usize) -> bool {
        self.segments[segment]
            .iter()
            .all(|item| item.inner.is_none())
    }

    pub fn new(default: Option<T>) -> Self {
        Self {
            segments: vec![Self::new_sealed_segment(default.clone(), 0)],
            default,
            freelist_head: 0,
        }
    }

    #[inline(always)]
    pub fn get_item_mut(&mut self, abs: ArenaIndex) -> &mut ArenaItem<T> {
        &mut self.segments[abs / MEMMAX][abs % MEMMAX]
    }

    #[inline(always)]
    pub fn get_item(&self, abs: ArenaIndex) -> &ArenaItem<T> {
        &self.segments[abs / MEMMAX][abs % MEMMAX]
    }

    #[inline(always)]
    pub fn reclaim(&mut self, abs: ArenaIndex) {
        let item = &mut self.segments[abs / MEMMAX][abs % MEMMAX];

        if item.inner.is_none() {
            return;
        }

        item.next_free = self.freelist_head;
        item.inner = None;
        self.freelist_head = abs;
    }

    fn add_segment(&mut self) {
        let old_tail = self.segments.len() * MEMMAX - 1;
        
        self.segments.push(Self::new_sealed_segment(self.default.clone(), self.segments.len() * MEMMAX));

        let new_head = old_tail + 1;

        self.segments[old_tail / MEMMAX][old_tail % MEMMAX].next_free = new_head;
        self.freelist_head = new_head;
    }

    #[inline(always)]
    pub fn push(&mut self, v: T) -> ArenaIndex {
        let mut head = self.freelist_head;

        if head == usize::MAX {
            self.add_segment();
            head = self.freelist_head;
        }

        let item = &mut self.segments[head / MEMMAX][head % MEMMAX];
        assert!(
            item.inner.is_none(),
            "allocating already-used slot {}",
            head
        );

        item.inner = Some(v);

        self.freelist_head = item.next_free;

        head
    }

    #[inline(always)]
    pub fn get(&self, index: ArenaIndex) -> &T {
        self.segments
            [index / 1024]
            [index % 1024]
            .inner
            .as_ref()
            .unwrap()
    }

    #[inline(always)]
    pub fn get_mut(&mut self, index: ArenaIndex) -> &mut T {
        self.segments
            [index / 1024]
            [index % 1024]
            .inner
            .as_mut()
            .unwrap()
    }

    #[inline(always)]
    pub unsafe fn raw_get(&self, index: ArenaIndex) -> *const T {
        &raw const *self.segments
            [index / 1024]
            [index % 1024]
            .inner
            .as_ref()
            .unwrap()
    }

    #[inline(always)]
    pub unsafe fn raw_get_mut(&mut self, index: ArenaIndex) -> *mut T {
        &raw mut *self.segments
            [index / 1024]
            [index % 1024]
            .inner
            .as_mut()
            .unwrap()
    }
}