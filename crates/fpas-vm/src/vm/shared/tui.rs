//! Shared `Std.Tui` session state and Turbo Vision facade records.
//!
//! **Documentation:** `docs/pascal/std/tui/app/vm-bridge.md`

use fpas_bytecode::Value;
use fpas_std::TuiSession;
use std::collections::{HashMap, VecDeque};
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
    /// Set by `Application.Quit`; consumed by the Turbo Vision run loop.
    pub quit_requested: bool,
    /// Turbo Vision backed handles for the `Std.Tui` facade.
    pub turbo_vision: TurboVisionState,
}

pub(crate) struct TurboVisionState {
    pub next_handle: u32,
    pub objects: HashMap<u32, TurboVisionObject>,
    pub menu_bar: Option<u32>,
    pub status_line: Option<u32>,
    pub pending_commands: VecDeque<u16>,
    pub quit_requested: bool,
    /// Headless override consumed by the next `RunFileDialog` call.
    pub test_file_dialog_result: Option<Option<String>>,
    /// Headless override consumed by the next `ExecDialog` call (closing command id).
    pub test_dialog_result: Option<i64>,
    /// FPAS-side widget tree changed since the last reconcile step.
    pub pending_reconcile: crate::vm::turbo_vision_bool_cell::TurboVisionBoolCell,
}

impl Default for TurboVisionState {
    fn default() -> Self {
        Self {
            next_handle: 1,
            objects: HashMap::new(),
            menu_bar: None,
            status_line: None,
            pending_commands: VecDeque::new(),
            quit_requested: false,
            test_file_dialog_result: None,
            test_dialog_result: None,
            pending_reconcile: crate::vm::turbo_vision_bool_cell::TurboVisionBoolCell::new(false),
        }
    }
}

impl fmt::Debug for TurboVisionState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("TurboVisionState")
            .field("next_handle", &self.next_handle)
            .field("object_count", &self.objects.len())
            .field("pending_commands", &self.pending_commands)
            .field("quit_requested", &self.quit_requested)
            .finish()
    }
}

pub(crate) enum TurboVisionObject {
    Dialog(TurboVisionDialog),
    Window(TurboVisionWindow),
    Button(TurboVisionButton),
    StaticText(TurboVisionStaticText),
    Memo(TurboVisionMemo),
    TextViewer(TurboVisionTextViewer),
    InputLine(TurboVisionInputLine),
    ListBox(TurboVisionListBox),
    CheckBox(TurboVisionCheckBox),
    RadioButton(TurboVisionRadioButton),
    MenuBar(TurboVisionMenuBar),
    StatusLine(TurboVisionStatusLine),
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct TurboVisionRect {
    pub x: i16,
    pub y: i16,
    pub width: i16,
    pub height: i16,
}

pub(crate) struct TurboVisionDialog {
    pub bounds: TurboVisionRect,
    pub title: String,
    pub children: Vec<u32>,
}

pub(crate) struct TurboVisionWindow {
    pub bounds: TurboVisionRect,
    pub title: String,
    pub children: Vec<u32>,
    pub on_desktop: bool,
}

#[derive(Clone)]
pub(crate) struct TurboVisionButton {
    pub bounds: TurboVisionRect,
    pub text: String,
    pub command_id: u16,
    pub attached: bool,
}

#[derive(Clone)]
pub(crate) struct TurboVisionStaticText {
    pub bounds: TurboVisionRect,
    pub text: String,
    pub attached: bool,
}

#[derive(Clone)]
pub(crate) struct TurboVisionMemo {
    pub bounds: TurboVisionRect,
    pub text: String,
    pub attached: bool,
}

#[derive(Clone)]
pub(crate) struct TurboVisionTextViewer {
    pub bounds: TurboVisionRect,
    pub text: String,
    pub attached: bool,
}

#[derive(Clone)]
pub(crate) struct TurboVisionInputLine {
    pub bounds: TurboVisionRect,
    pub max_length: usize,
    pub text_cell: crate::vm::turbo_vision_input_text_cell::TurboVisionInputTextCell,
    pub attached: bool,
}

#[derive(Clone)]
pub(crate) struct TurboVisionListBox {
    pub bounds: TurboVisionRect,
    pub items: Vec<String>,
    pub command_id: u16,
    pub selection_cell: crate::vm::turbo_vision_list_selection_cell::TurboVisionListSelectionCell,
    pub attached: bool,
}

#[derive(Clone)]
pub(crate) struct TurboVisionCheckBox {
    pub bounds: TurboVisionRect,
    pub text: String,
    pub checked_cell: crate::vm::turbo_vision_bool_cell::TurboVisionBoolCell,
    pub attached: bool,
}

#[derive(Clone)]
pub(crate) struct TurboVisionRadioButton {
    pub bounds: TurboVisionRect,
    pub text: String,
    pub group_id: u16,
    pub selected_cell: crate::vm::turbo_vision_bool_cell::TurboVisionBoolCell,
    pub attached: bool,
}

#[derive(Clone)]
pub(crate) struct TurboVisionMenuBar {
    pub bounds: TurboVisionRect,
    pub menus: Vec<TurboVisionMenu>,
    pub attached: bool,
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
pub(crate) struct TurboVisionStatusLine {
    pub bounds: TurboVisionRect,
    pub items: Vec<TurboVisionStatusItem>,
    pub attached: bool,
}

#[derive(Clone)]
pub(crate) struct TurboVisionStatusItem {
    pub text: String,
    pub key_code: u16,
    pub command_id: u16,
}
