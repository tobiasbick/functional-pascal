//! Pascal-facing command identifiers for Turbo Vision widgets and callbacks.
//!
//! **Documentation:** `docs/pascal/std/tui/app/types.md`

/// Standard dialog accept command (`Command.Accept`; value `1`).
pub const COMMAND_OK: i64 = 1;
/// Standard dialog cancel command (`Command.Cancel`).
pub const COMMAND_CANCEL: i64 = 2;
/// Close the source view or window (`Command.Close`).
pub const COMMAND_CLOSE: i64 = 3;
/// Exit the application (`Command.Quit`).
pub const COMMAND_QUIT: i64 = 4;
