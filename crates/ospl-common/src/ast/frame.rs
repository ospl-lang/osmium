/// Represents a frame on the callstack.
#[derive(Debug, Default, Clone, PartialEq)]
#[derive(serde::Serialize, serde::Deserialize)]
pub struct RuntimeFrame {
    /// Stores indexes into the arena
    pub indexes: Vec<usize>,
}

impl RuntimeFrame {
    pub fn new(
        indexes: Vec<usize>,
    ) -> Self {
        return Self { indexes };
    }
}