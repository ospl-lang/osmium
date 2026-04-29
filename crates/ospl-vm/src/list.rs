use crate::arena::ArenaIndex;

#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct List {
    pub inner: Vec<ArenaIndex>
}
