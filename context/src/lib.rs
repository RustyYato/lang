mod ctx;

mod ptr;

pub mod types;

mod utils;

pub use ctx::{AllocContext, Context, ContextId, TypeContext};

pub struct TargetSpec {
    pub pointer_size_bytes: u8,
    pub pointer_align_log2: u8,
    pub pointer_diff_size_bytes: u8,
    pub pointer_diff_align_log2: u8,
}

#[cfg(test)]
const TEST_TARGET_SPEC: TargetSpec = TargetSpec {
    pointer_size_bytes: 8,
    pointer_align_log2: 3,
    pointer_diff_size_bytes: 8,
    pointer_diff_align_log2: 3,
};
