//! Pure copy-on-write sequence structure transformations.

mod array;
mod model;
mod string;

pub(in crate::vm::debug) use array::{insert as insert_array, remove as remove_array};
pub(in crate::vm::debug) use string::replace_character as replace_string_character;
