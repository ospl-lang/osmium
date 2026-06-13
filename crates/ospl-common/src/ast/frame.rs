/// Represents a frame on the callstack.
#[derive(Debug, Clone, PartialEq)]
#[derive(serde::Serialize, serde::Deserialize)]
#[repr(C)]
pub struct RuntimeFrame {
    /// Stores indexes into the arena
    pub indexes: Vec<usize>,
}

impl RuntimeFrame {
    pub fn new(
        indexes: Vec<usize>,
    ) -> Self {
        return Self {
            indexes,
        };
    }
}

impl Default for RuntimeFrame {
    fn default() -> Self {
        return Self {
            indexes: Vec::with_capacity(32),
        }
    }
}
