//! Standard-library declarations shared by semantic analysis and execution.
//!
//! This crate describes names, field shapes, and discriminant order. It does not
//! create runtime values, perform host operations, or depend on the interpreter.

mod aggregates;
mod console;
/// Discriminant names and lookup for standard console keys.
pub mod key_event;
/// Nominal error families and their declared payloads.
pub mod native_errors;
/// Detached native record declarations.
pub mod native_records;
mod std_units;

pub use aggregates::{RUNTIME_AGGREGATE_TYPES, is_runtime_aggregate_type};
pub use console::{
    CONSOLE_COLOR_KIND_VARIANTS, EVENT_KIND_VARIANTS, MOUSE_ACTION_VARIANTS, MOUSE_BUTTON_VARIANTS,
    event_kind_index, mouse_action_index, mouse_button_index,
};
pub use key_event::{KEY_KIND_VARIANTS, key_kind_index};
pub use std_units::*;

fn variant_index(variants: &[&str], name: &str) -> Option<usize> {
    variants.iter().position(|&variant| variant == name)
}
