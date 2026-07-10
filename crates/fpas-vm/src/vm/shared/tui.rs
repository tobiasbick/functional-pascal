//! Shared `Std.Tui` session state and try-2 record helpers.
//!
//! **Documentation:** `docs/pascal/std/tui/app/vm-bridge.md`

use fpas_bytecode::Value;
use fpas_std::TuiSession;
use std::fmt;

#[derive(Debug, Default)]
pub(crate) struct TuiState {
    pub session: TuiSession,
    /// Turbo Vision `Application.OnCommand`: `procedure (Application, integer)`.
    pub on_command: Option<Value>,
    /// Turbo Vision `Application.OnKey`: `function (Application, Std.Console.KeyEvent): boolean`.
    pub turbo_vision_on_key: Option<Value>,
    /// Turbo Vision `Application.OnMouse`: `procedure (Application, Std.Console.Event)`.
    pub turbo_vision_on_mouse: Option<Value>,
    /// Set by `Application.Quit`; consumed by the try-2 run loop.
    pub quit_requested: bool,
    /// Headless modal stub state (`TestSetDialogResult`, `MessageBox` queue).
    pub turbo_vision: TurboVisionState,
}

/// Remaining headless test hooks not yet owned by the try-2 session.
pub(crate) struct TurboVisionState {
    /// Headless override consumed by the next `MessageBox` call (closing command id).
    pub test_dialog_result: Option<i64>,
}

impl Default for TurboVisionState {
    fn default() -> Self {
        Self {
            test_dialog_result: None,
        }
    }
}

impl fmt::Debug for TurboVisionState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("TurboVisionState")
            .field("test_dialog_result", &self.test_dialog_result)
            .finish()
    }
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct TurboVisionRect {
    pub x: i16,
    pub y: i16,
    pub width: i16,
    pub height: i16,
}

#[derive(Clone)]
pub(crate) struct TurboVisionMenu {
    pub title: String,
    pub items: Vec<TurboVisionMenuItem>,
}

#[derive(Clone)]
pub(crate) struct TurboVisionMenuItem {
    pub text: String,
    pub command_id: u16,
}

#[derive(Clone)]
pub(crate) struct TurboVisionStatusItem {
    pub text: String,
    pub key_code: u16,
    pub command_id: u16,
}

/// One node in an FPAS outline tree.
#[derive(Clone, Debug)]
pub(crate) struct TurboVisionOutlineNode {
    pub text: String,
    pub children: Vec<TurboVisionOutlineNode>,
    pub expanded: bool,
}
