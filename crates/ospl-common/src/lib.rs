pub mod ast;
pub mod inst;
pub mod nicities;

pub mod types {
    /// An absolute address with no indirection
    pub type AbsAddress = usize;

    /// A frame address with 
    pub type FrameAddress = usize;
}