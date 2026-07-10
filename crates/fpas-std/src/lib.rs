#![cfg_attr(
    test,
    expect(
        clippy::expect_used,
        clippy::panic,
        reason = "runtime tests use expect/panic to keep fixture assertions compact"
    )
)]
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
mod env;
mod error;
mod fs;
mod graph;
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
mod tui;
mod ui;

pub use console::{
    CapturedOutput, Console, KeyInput, ReadLnQueue, ScreenSnapshot, TextInput,
    read_line_from_stdin, validate_packed_crt_color,
};
pub use console_event::{
    ConsoleEvent, EVENT_KIND_VARIANTS, MOUSE_ACTION_VARIANTS, MOUSE_BUTTON_VARIANTS,
    event_kind_index, mouse_action_index, mouse_button_index,
};
pub use error::StdError;
pub use graph::{
    GRAPH_EVENT_KIND_VARIANTS, GRAPH_EXIT_REASON_VARIANTS, GraphEvent, GraphEventKind, GraphHost,
    GraphSession, HeadlessGraphTestModeGuard, UploadedFrame, headless_graph_test_depth_for_tests,
    last_headless_graph_frame_for_tests, pop_headless_graph_test_mode,
    push_headless_graph_test_mode, with_headless_graph_backend_for_tests,
};
pub use intrinsics::run_intrinsic;
pub use key_event::{ConsoleKeyEvent, KEY_KIND_VARIANTS, key_kind_index};
pub use std_units::{
    STD_UNIT_ARGS, STD_UNIT_ARRAY, STD_UNIT_CONSOLE, STD_UNIT_CONV, STD_UNIT_DICT, STD_UNIT_ENV,
    STD_UNIT_FS, STD_UNIT_GRAPH, STD_UNIT_JSON, STD_UNIT_MATH, STD_UNIT_OPTION, STD_UNIT_PARSE,
    STD_UNIT_PATH, STD_UNIT_PROC, STD_UNIT_RANDOM, STD_UNIT_RESULT, STD_UNIT_STR, STD_UNIT_TASK,
    STD_UNIT_TEST, STD_UNIT_TIME, STD_UNIT_TUI, STD_UNITS_KNOWN, canonical_std_unit_from_segments,
    canonical_std_unit_from_tail, is_std_root_segment, std_symbols, std_unit_symbols,
    std_units_list_for_hint,
};
pub use test::{assert_screen_cell, assert_screen_line, reset_test_skip_state, test_was_skipped};
pub use tui::{
    BlockedInput, CM_ABOUT, CM_CANCEL, CM_CLOSE, CM_OK, CM_OPEN, CM_QUIT, CM_USER,
    COMMAND_ID_CLOSE, COMMAND_ID_NEXT_WINDOW, COMMAND_ID_ZOOM, COMMAND_ID_ZOOM_BACK, CommandEvent,
    CommandId, CommandKind, CommandRegistry, DamageRegion, FocusDirection,
    MESSAGE_BOX_OPTION_ABOUT, MESSAGE_BOX_OPTION_CANCEL_BUTTON, MESSAGE_BOX_OPTION_CONFIRMATION,
    MESSAGE_BOX_OPTION_ERROR, MESSAGE_BOX_OPTION_INFORMATION, MESSAGE_BOX_OPTION_NO_BUTTON,
    MESSAGE_BOX_OPTION_OK_BUTTON, MESSAGE_BOX_OPTION_OK_CANCEL, MESSAGE_BOX_OPTION_WARNING,
    MESSAGE_BOX_OPTION_YES_BUTTON, MESSAGE_BOX_OPTION_YES_NO_CANCEL, ProcessOutcome,
    TUI_EVENT_KIND_VARIANTS, TUI_EXIT_REASON_VARIANTS, TuiEvent, TuiHost, TuiSession, ViewId,
    ViewRect,
};
pub use ui::{UiEvent, UiHost, UiHostSurface, UiModifiers, UiMouse, UiResize, UiWheel};

/// Returns the index of `name` in `variants`, or 0 if not found.
///
/// Used by all console enum variant name → index conversions.
pub(crate) fn variant_index(variants: &[&str], name: &str) -> usize {
    variants.iter().position(|&v| v == name).unwrap_or(0)
}
