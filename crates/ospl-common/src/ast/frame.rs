/// Represents a frame on the callstack.
#[derive(Debug, Default, Clone, PartialEq)]
pub struct RuntimeFrame {
    /// Stores indexes into the arena
    pub indexes: Vec<usize>,
    pub num_args: usize,
    pub num_captures: usize,
}

impl RuntimeFrame {
    pub fn new(
        indexes: Vec<usize>,
        num_args: usize,
        num_captures: usize
    ) -> Self {
        return Self { indexes, num_args, num_captures };
    }
}