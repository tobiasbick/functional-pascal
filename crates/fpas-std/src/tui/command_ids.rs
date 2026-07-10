//! Internal command id values shared by the compiler and VM.
//!
//! Pascal programs use `CM_*` constants from `Std.Tui` (`CM_OK`, `CM_QUIT`, …).
//!
//! **Documentation:** `docs/pascal/std/tui/app/types.md`

/// Borland `cmOK` / `CM_OK`.
pub const COMMAND_OK: i64 = 10;
/// Borland `cmCancel` / `CM_CANCEL`.
pub const COMMAND_CANCEL: i64 = 11;
/// Borland `cmClose` / `CM_CLOSE`.
pub const COMMAND_CLOSE: i64 = 4;
/// Borland `cmQuit` / `CM_QUIT`.
pub const COMMAND_QUIT: i64 = 1;
