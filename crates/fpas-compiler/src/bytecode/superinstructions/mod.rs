//! Compiler selection of integer superinstructions.

mod branches;
mod immediates;

pub(super) use branches::fuse_integer_branch;
pub(super) use immediates::integer_immediate;
