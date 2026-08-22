#![cfg_attr(
    test,
    allow(
        clippy::expect_used,
        clippy::panic,
        clippy::unwrap_used,
        reason = "runtime tests use unwrap/expect/panic to keep fixture assertions compact"
    )
)]

//! FPAS standard-library runtime (`Std.*` procedures backed by intrinsics and console I/O).
//!
//! **Documentation:** `docs/pascal/std/README.md` and per-unit files under `docs/pascal/std/` (from the repository root).
//! **Maintenance:** Keep those Markdown files aligned with this crate, `fpas-vm`, `fpas-compiler`, `fpas-bytecode`, and `fpas-sema` `std_registry.rs`.

mod aggregate_factory;
mod array;
mod console;
mod console_event;
mod conv;
mod dict;
mod env;
mod error;
mod fs;
mod intrinsic_args;
mod intrinsics;
mod json;
pub mod key_event;
mod limits;
mod math;
mod numeric_text;
mod parse;
mod path;
mod proc;
mod random;
mod result_option;
mod std_units;
mod str;
mod test;
mod text;
mod time;
mod toml;

pub use aggregate_factory::{AggregateFactory, RUNTIME_AGGREGATE_TYPES};
pub use console::{
    CONSOLE_COLOR_KIND_VARIANTS, CapturedOutput, Console, ConsoleCell, ConsoleColor, ConsoleRect,
    KeyInput, ReadLnQueue, SavedRegionId, ScreenSnapshot, TextInput, read_line_from_stdin,
    validate_packed_crt_color,
};
pub use console_event::{
    ConsoleEvent, EVENT_KIND_VARIANTS, MOUSE_ACTION_VARIANTS, MOUSE_BUTTON_VARIANTS,
    event_kind_index, mouse_action_index, mouse_button_index,
};
pub use error::StdError;
#[cfg(test)]
pub(crate) use intrinsics::execute_test_intrinsic;
pub use intrinsics::run_intrinsic_borrowed;
pub use key_event::{ConsoleKeyEvent, KEY_KIND_VARIANTS, key_kind_index};
pub use std_units::{
    STD_UNIT_ARGS, STD_UNIT_ARRAY, STD_UNIT_CONSOLE, STD_UNIT_CONV, STD_UNIT_DICT, STD_UNIT_ENV,
    STD_UNIT_FS, STD_UNIT_JSON, STD_UNIT_MATH, STD_UNIT_OPTION, STD_UNIT_PARSE, STD_UNIT_PATH,
    STD_UNIT_PROC, STD_UNIT_RANDOM, STD_UNIT_RESULT, STD_UNIT_STR, STD_UNIT_TASK, STD_UNIT_TEST,
    STD_UNIT_TIME, STD_UNIT_TOML, STD_UNIT_TUI, STD_UNITS_INTRINSIC, STD_UNITS_KNOWN,
    canonical_std_unit_from_segments, canonical_std_unit_from_tail, is_std_root_segment,
    std_symbols, std_unit_symbols, std_units_list_for_hint,
};
pub use test::{assert_screen_cell, assert_screen_line, reset_test_skip_state, test_was_skipped};

/// Returns the index of `name` in `variants`, or `None` if not found.
pub(crate) fn variant_index(variants: &[&str], name: &str) -> Option<usize> {
    variants.iter().position(|&v| v == name)
}
