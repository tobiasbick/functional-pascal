//! Maps every source comment to formatter emission anchors.

mod anchors;
mod emit;
mod map;
mod traversal;

pub(crate) use anchors::stmt_start;
pub(crate) use emit::{emit_leading_comments, emit_trailing_comments, emit_trailing_end_comments};
pub use map::CommentMap;
