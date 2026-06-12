//! Maps preservable leading comments to declaration anchor offsets.

mod emit;
mod map;

pub(crate) use emit::emit_leading_comments;
pub use map::CommentMap;
