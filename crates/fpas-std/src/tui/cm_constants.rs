//! Borland / turbo-vision `CM_*` command identifiers for try-2 `Std.Tui`.
//!
//! Values match `turbo_vision::core::command` at tag `v2.0.0`. Pascal-facing
//! symbols will use these names directly (`CM_QUIT`, not `Command.Quit`).
//!
//! **Documentation:** `docs/refactor-tui-try-2/target-api.md`

#![allow(dead_code, reason = "try-2 constants; wired in sema during phase 1/2")]

// Modal dialog control
/// Modal dialog continues (no end command yet).
pub const CM_CONTINUE: i64 = 0;

// Standard commands (Borland views.h)
/// Exit the application (Borland `cmQuit`).
pub const CM_QUIT: i64 = 1;
/// Close the source view or window (Borland `cmClose`).
pub const CM_CLOSE: i64 = 4;
/// Zoom window (Borland `cmZoom`).
pub const CM_ZOOM: i64 = 5;
/// Keyboard move/resize mode (Borland `cmResize`).
pub const CM_RESIZE: i64 = 6;
/// Cycle to next window (Borland `cmNext`).
pub const CM_NEXT: i64 = 7;
/// Cycle to previous window (Borland `cmPrev`).
pub const CM_PREV: i64 = 8;
/// Dialog OK (Borland `cmOK`).
pub const CM_OK: i64 = 10;
/// Dialog cancel (Borland `cmCancel`).
pub const CM_CANCEL: i64 = 11;
/// Yes button (Borland `cmYes`).
pub const CM_YES: i64 = 12;
/// No button (Borland `cmNo`).
pub const CM_NO: i64 = 13;
/// Activate default button (Borland `cmDefault`).
pub const CM_DEFAULT: i64 = 14;

// Edit / window
pub const CM_CUT: i64 = 20;
pub const CM_COPY: i64 = 21;
pub const CM_PASTE: i64 = 22;
pub const CM_UNDO: i64 = 23;
pub const CM_CLEAR: i64 = 24;
pub const CM_TILE: i64 = 25;
pub const CM_CASCADE: i64 = 26;

// Broadcast / internal
pub const CM_RECEIVED_FOCUS: i64 = 50;
pub const CM_RELEASED_FOCUS: i64 = 51;
pub const CM_COMMAND_SET_CHANGED: i64 = 52;
pub const CM_SELECT_WINDOW_NUM: i64 = 55;
pub const CM_SCROLLBAR_CHANGED: i64 = 57;
pub const CM_RECORD_HISTORY: i64 = 60;
pub const CM_GRAB_DEFAULT: i64 = 61;
pub const CM_RELEASE_DEFAULT: i64 = 62;
pub const CM_REDRAW: i64 = 63;
pub const CM_FOCUS_LINK: i64 = 66;
pub const CM_RADIO_SELECTED: i64 = 67;
pub const CM_SHOW_HISTORY: i64 = 69;
pub const CM_HISTORY_SELECTED: i64 = 70;

// Port-specific
pub const CM_SCREENSHOT: i64 = 31;

// Help / about
pub const CM_ABOUT: i64 = 100;
pub const CM_HELP_INDEX: i64 = 140;

// File menu
pub const CM_NEW: i64 = 300;
pub const CM_OPEN: i64 = 301;
pub const CM_SAVE: i64 = 302;
pub const CM_SAVE_AS: i64 = 303;
pub const CM_SAVE_ALL: i64 = 304;
pub const CM_CLOSE_FILE: i64 = 305;

/// Suggested base for application-private commands (try-2 convention).
pub const CM_USER: i64 = 4096;
