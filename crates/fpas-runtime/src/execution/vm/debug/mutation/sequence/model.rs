//! Detached sequence mutation results prepared for atomic commit.

use crate::Value;

/// Detached array value and optional operation metadata prepared for commit.
#[derive(Debug)]
pub(in crate::execution::vm::debug) struct ArrayTransformation {
    pub array: Value,
    pub index: usize,
    pub removed: Option<Value>,
}

/// Detached string value and character metadata prepared for commit.
#[derive(Debug)]
pub(in crate::execution::vm::debug) struct StringTransformation {
    pub string: Value,
    pub index: usize,
    pub old_character: Value,
    pub new_character: Value,
}
