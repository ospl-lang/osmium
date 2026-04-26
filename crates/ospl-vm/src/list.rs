use crate::{RuntimeValue, arena::{Arena, ArenaIndex}, gc::GcEvent};

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

    pub fn pop_mut<'a>(&mut self, arena: &'a mut Arena) -> Option<&'a mut RuntimeValue> {
        let idx = self.inner.pop()?;

        arena.gc_event(GcEvent::EndReference { to: idx });
        let item = arena.get_mut(idx);

        return Some(item);
    }
}
