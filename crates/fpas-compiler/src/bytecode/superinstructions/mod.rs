//! Compiler selection of integer superinstructions.

mod branches;
mod immediates;
mod tail_calls;

pub(super) use branches::fuse_integer_branch;
pub(crate) use immediates::integer_immediate;
pub(super) use tail_calls::convert_tail_call;
