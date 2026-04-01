#![cfg_attr(
    test,
    expect(
        clippy::unwrap_used,
        reason = "runtime tests use unwrap to keep console fixture assertions compact"
    )
)]

//! FPAS standard-library runtime (`Std.*` procedures backed by intrinsics and console I/O).
//!
//! **Documentation:** `docs/pascal/std/README.md` and per-unit files under `docs/pascal/std/` (from the repository root).
//! **Maintenance:** Keep those Markdown files aligned with this crate, `fpas-vm`, `fpas-compiler`, `fpas-bytecode`, and `fpas-sema` `std_registry.rs`.

mod array;
mod console;
mod console_event;
mod conv;
mod dict;
mod error;
mod helpers;
mod intrinsics;
pub mod key_event;
mod math;
mod result_option;
mod std_units;
mod str;

pub use console::{
    CapturedOutput, Console, KeyInput, ReadLnQueue, TextInput, read_line_from_stdin,
};
pub use console_event::{
    ConsoleEvent, EVENT_KIND_VARIANTS, MOUSE_ACTION_VARIANTS, MOUSE_BUTTON_VARIANTS,
    event_kind_index, mouse_action_index, mouse_button_index,
};
pub use error::StdError;
pub use intrinsics::run_intrinsic;
pub use key_event::{ConsoleKeyEvent, KEY_KIND_VARIANTS};
pub use std_units::{
    STD_UNIT_ARRAY, STD_UNIT_CONSOLE, STD_UNIT_CONV, STD_UNIT_DICT, STD_UNIT_MATH, STD_UNIT_OPTION,
    STD_UNIT_RESULT, STD_UNIT_STR, STD_UNIT_TASK, STD_UNITS_KNOWN,
    canonical_std_unit_from_segments, canonical_std_unit_from_tail, is_std_root_segment,
    std_symbols, std_unit_symbols, std_units_list_for_hint,
};

/// Returns the index of `name` in `variants`, or 0 if not found.
///
/// Used by all console enum variant name → index conversions.
pub(crate) fn variant_index(variants: &[&str], name: &str) -> usize {
    variants.iter().position(|&v| v == name).unwrap_or(0)
}
