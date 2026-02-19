use crate::{arena::{Arena, ArenaIndex}, gc::GcEvent};

#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct List {
    pub inner: Vec<ArenaIndex>
}

impl List {
    pub fn push(&mut self, idx: ArenaIndex, arena: &mut Arena) {
        self.inner.push(idx);
        arena.gc_event(GcEvent::NewReference {
            to: idx
        });
    }
}
