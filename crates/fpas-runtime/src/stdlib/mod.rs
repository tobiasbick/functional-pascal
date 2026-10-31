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
//! **Maintenance:** Shared declarations belong to `fpas-builtins`; executable
//! implementations belong here and in the runtime's host integration modules.

mod acp;
mod aggregate_factory;
mod array;
/// Shared `Std.Math.Bits` primitives; see `docs/pascal/std/numeric/bits.md`.
pub mod bits;
mod boolean_text;
mod bytes;
mod console;
mod console_event;
mod conv;
mod dict;
mod encoding;
mod env;
pub(crate) mod error;
pub(crate) mod fs;
pub use fs::{read_text_for_host, write_text_for_host};
mod intrinsic_args;
mod intrinsics;
mod json;
pub mod key_event;
mod limits;
mod math;
/// Shared typed operational error catalog and value builders.
pub mod native_errors;
pub use fpas_builtins::native_records;
mod parse;
mod path;
mod proc;
mod result_option;
mod str;
mod test;
mod text;
mod time;
mod toml;

pub use aggregate_factory::AggregateFactory;
pub use array::sorting::sort_by_cached_keys;
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
pub use fpas_builtins::{RUNTIME_AGGREGATE_TYPES, is_runtime_aggregate_type};
pub use fpas_builtins::{
    STD_UNIT_ACP, STD_UNIT_ACP_EXPERIMENTAL, STD_UNIT_ACP_HOST, STD_UNIT_ACP_TYPES,
    STD_UNIT_ACP_V2, STD_UNIT_ARGS, STD_UNIT_ARRAY, STD_UNIT_BITS, STD_UNIT_CALENDAR, STD_UNIT_CLI,
    STD_UNIT_CONSOLE, STD_UNIT_CRYPTO, STD_UNIT_DICT, STD_UNIT_ENCODING, STD_UNIT_ENV,
    STD_UNIT_ERRORS, STD_UNIT_FORMAT, STD_UNIT_FS, STD_UNIT_JSON, STD_UNIT_MATH, STD_UNIT_MCP,
    STD_UNIT_MCP_TYPES, STD_UNIT_NET, STD_UNIT_OPTION, STD_UNIT_PARSE, STD_UNIT_PATH,
    STD_UNIT_PROC, STD_UNIT_RANDOM, STD_UNIT_RESULT, STD_UNIT_SERVER, STD_UNIT_STR, STD_UNIT_TASK,
    STD_UNIT_TEST, STD_UNIT_TIME, STD_UNIT_TOML, STD_UNIT_TUI, STD_UNITS_INTRINSIC,
    STD_UNITS_KNOWN, canonical_std_unit_from_segments, canonical_std_unit_from_tail,
    is_retired_type_helper_unit, is_std_root_segment, std_symbols, std_unit_symbols,
    std_units_list_for_hint,
};
#[cfg(test)]
pub(crate) use intrinsics::execute_test_intrinsic;
pub use intrinsics::run_intrinsic_borrowed;
pub use key_event::{ConsoleKeyEvent, KEY_KIND_VARIANTS, key_kind_index};
pub use proc::ProgramStderr;
pub use test::{assert_screen_cell, assert_screen_line, reset_test_skip_state, test_was_skipped};
