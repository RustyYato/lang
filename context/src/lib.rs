mod ctx;

mod ptr;

pub mod types;

pub use ctx::{AllocContext, Context, ContextId, TypeContext};

pub struct TargetSpec {
    pub pointer_size_bytes: u8,
    pub pointer_align_log2: u8,
    pub pointer_diff_size_bytes: u8,
    pub pointer_diff_align_log2: u8,
}
