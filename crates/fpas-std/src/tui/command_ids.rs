//! Pascal-facing command identifiers for Turbo Vision widgets and callbacks.
//!
//! Values match Borland Turbo Vision / `turbo-vision` 2.0 `CM_*` constants.
//!
//! **Documentation:** `docs/pascal/std/tui/app/types.md`

/// Standard dialog accept command (`Command.Accept`; Borland `cmOK`).
pub const COMMAND_OK: i64 = 10;
/// Standard dialog cancel command (`Command.Cancel`; Borland `cmCancel`).
pub const COMMAND_CANCEL: i64 = 11;
/// Close the source view or window (`Command.Close`; Borland `cmClose`).
pub const COMMAND_CLOSE: i64 = 4;
/// Exit the application (`Command.Quit`; Borland `cmQuit`).
pub const COMMAND_QUIT: i64 = 1;
