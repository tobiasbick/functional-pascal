//! TUI session state: view registry and owned widget roots.
//!
//! **Documentation:** `docs/pascal/std/tui/app/vm-bridge.md`

use super::registry::{ViewKind, ViewRegistry};
use crate::vm::shared::{
    TurboVisionMenu, TurboVisionOutlineNode, TurboVisionRect, TurboVisionStatusItem,
};
use crate::vm::turbo_vision_bool_cell::TurboVisionBoolCell;
use crate::vm::turbo_vision_input_text_cell::TurboVisionInputTextCell;
use crate::vm::turbo_vision_list_selection_cell::TurboVisionListSelectionCell;
use std::cell::RefCell;
use std::collections::HashMap;
use std::collections::HashSet;
use std::rc::Rc;
use turbo_vision::core::geometry::{Point, Rect};
use turbo_vision::views::View;
use turbo_vision::views::dialog::Dialog;
use turbo_vision::views::edit_window::EditWindow;
use turbo_vision::views::window::Window;

/// Button constructed by `Button.New` and not yet attached to a parent.
pub(crate) struct DetachedButton {
    pub button: Box<dyn View>,
    pub local_bounds: Rect,
}

/// Static text constructed by `StaticText.New` and not yet attached to a parent.
pub(crate) struct DetachedStaticText {
    pub static_text: Box<dyn View>,
    pub local_bounds: Rect,
}

/// Host-side button state retained after attach.
pub(crate) struct TuiButtonState {
    pub command: u16,
    pub is_default: bool,
    pub text: String,
}

/// Check box constructed by `CheckBox.New` and not yet attached to a parent.
pub(crate) struct DetachedCheckBox {
    pub check_box: Box<dyn View>,
    pub local_bounds: Rect,
}

/// Input line constructed by `InputLine.New` and not yet attached to a parent.
pub(crate) struct DetachedInputLine {
    pub local_bounds: Rect,
    pub text_cell: TurboVisionInputTextCell,
    pub max_length: usize,
}

/// List box constructed by `ListBox.New` and not yet attached to a parent.
pub(crate) struct DetachedListBox {
    pub list_box: Box<dyn View>,
}

/// Outline constructed by `Outline.New` and not yet attached to a parent.
pub(crate) struct DetachedOutline {
    pub outline: Box<dyn View>,
}

/// Radio button constructed by `RadioButton.New` and not yet attached to a parent.
pub(crate) struct DetachedRadioButton {
    pub radio_button: Box<dyn View>,
    pub local_bounds: Rect,
}

/// Memo constructed by `Memo.New` and not yet attached to a parent.
pub(crate) struct DetachedMemo {
    pub memo: Box<dyn View>,
    pub local_bounds: Rect,
}

/// Text viewer constructed by `TextViewer.New` and not yet attached to a parent.
pub(crate) struct DetachedTextViewer {
    pub text_viewer: Box<dyn View>,
    pub local_bounds: Rect,
}

/// A headless screen-space target for a stateful control.
#[derive(Clone, Copy)]
pub(crate) struct MouseHitTarget {
    /// Opaque Turbo Vision handle for the target control.
    pub handle: u32,
    /// Screen-space rectangle that accepts the click.
    pub hit: Rect,
    /// Point delivered to Turbo Vision for the click.
    pub click: Point,
}

/// Host-side radio button state retained after attach.
#[derive(Clone)]
pub(crate) struct TuiRadioButtonState {
    pub bounds: Rect,
    pub text: String,
    pub group_id: u16,
    pub selected_cell: TurboVisionBoolCell,
}

/// Host-side list box state retained after attach.
pub(crate) struct TuiListBoxState {
    pub items: Vec<String>,
    pub selection_cell: TurboVisionListSelectionCell,
}

/// Host-side outline state retained after attach.
pub(crate) struct TuiOutlineState {
    pub roots: Vec<TurboVisionOutlineNode>,
    pub selection_cell: TurboVisionListSelectionCell,
}

/// Host-side input line state retained after attach.
pub(crate) struct TuiInputLineState {
    pub text_cell: TurboVisionInputTextCell,
    pub max_length: usize,
    pub view_binding: Option<Rc<RefCell<String>>>,
}

/// Menu bar data owned by Turbo Vision until attached via `Application.SetMenuBar`.
pub(crate) struct TuiMenuBarState {
    pub bounds: TurboVisionRect,
    pub menus: Vec<TurboVisionMenu>,
}

/// Status line data owned by Turbo Vision until attached via `Application.SetStatusLine`.
pub(crate) struct TuiStatusLineState {
    pub bounds: TurboVisionRect,
    pub items: Vec<TurboVisionStatusItem>,
}

/// Owned top-level widget waiting for modal exec or desktop attach.
pub(crate) enum TuiRoot {
    ModalDialog(Box<Dialog>),
    Window(Box<Window>),
    EditorWindow(Box<EditWindow>),
}

/// Rust-owned Turbo Vision session state.
#[derive(Default)]
pub(crate) struct TurboVisionSession {
    pub registry: ViewRegistry,
    session_open: bool,
    app_handle: Option<u32>,
    roots: HashMap<u32, TuiRoot>,
    /// Buttons from `Button.New` awaiting parent attach.
    detached_buttons: HashMap<u32, DetachedButton>,
    /// Static text from `StaticText.New` awaiting parent attach.
    detached_static_texts: HashMap<u32, DetachedStaticText>,
    /// Check boxes from `CheckBox.New` awaiting parent attach.
    detached_check_boxes: HashMap<u32, DetachedCheckBox>,
    /// Input lines from `InputLine.New` awaiting parent attach.
    detached_input_lines: HashMap<u32, DetachedInputLine>,
    /// List boxes from `ListBox.New` awaiting parent attach.
    detached_list_boxes: HashMap<u32, DetachedListBox>,
    /// Outlines from `Outline.New` awaiting parent attach.
    detached_outlines: HashMap<u32, DetachedOutline>,
    /// Radio buttons from `RadioButton.New` awaiting parent attach.
    detached_radio_buttons: HashMap<u32, DetachedRadioButton>,
    /// Memos from `Memo.New` awaiting parent attach.
    detached_memos: HashMap<u32, DetachedMemo>,
    /// Text viewers from `TextViewer.New` awaiting parent attach.
    detached_text_viewers: HashMap<u32, DetachedTextViewer>,
    /// Host-side check box state keyed by Turbo Vision handle.
    check_box_cells: HashMap<u32, TurboVisionBoolCell>,
    /// Host-side input line state keyed by Turbo Vision handle.
    input_line_states: HashMap<u32, TuiInputLineState>,
    /// Host-side list box state keyed by Turbo Vision handle.
    list_box_states: HashMap<u32, TuiListBoxState>,
    /// Host-side outline state keyed by Turbo Vision handle.
    outline_states: HashMap<u32, TuiOutlineState>,
    /// Host-side radio button state keyed by Turbo Vision handle.
    radio_button_states: HashMap<u32, TuiRadioButtonState>,
    /// Radio group members keyed by FPAS `GroupId`.
    radio_group_members: HashMap<u16, Vec<u32>>,
    /// Host-side button state keyed by Turbo Vision handle.
    button_states: HashMap<u32, TuiButtonState>,
    /// Host-side static text keyed by Turbo Vision handle.
    static_text_texts: HashMap<u32, String>,
    /// Host-side memo text keyed by Turbo Vision handle.
    memo_texts: HashMap<u32, String>,
    /// Host-side text viewer text keyed by Turbo Vision handle.
    text_viewer_texts: HashMap<u32, String>,
    /// Child handle to parent dialog/window handle after attach.
    child_parents: HashMap<u32, u32>,
    /// Headless test click targets keyed by Turbo Vision button handle.
    button_clicks: HashMap<u32, Point>,
    /// Screen-space mouse hit targets for check boxes and radio buttons.
    mouse_hit_targets: Vec<MouseHitTarget>,
    /// Stateful controls clicked through the headless test hook during the next run loop.
    pending_mouse_state_toggles: Vec<u32>,
    /// Window handles attached to the upstream desktop via `Desktop.Add`.
    desktop_windows: HashSet<u32>,
    menu_bars: HashMap<u32, TuiMenuBarState>,
    status_lines: HashMap<u32, TuiStatusLineState>,
    attached_menu_bar: Option<u32>,
    attached_status_line: Option<u32>,
    /// Headless file dialog result queued by the interim test helper.
    file_dialog_result: Option<Option<String>>,
    /// Headless modal stub consumed by the next `MessageBox` call (interim test helper).
    dialog_result: Option<i64>,
}

impl TurboVisionSession {
    /// Clears registry, roots, and session flags.
    pub fn reset(&mut self) {
        self.registry.clear();
        self.session_open = false;
        self.app_handle = None;
        self.roots.clear();
        self.detached_buttons.clear();
        self.detached_static_texts.clear();
        self.detached_check_boxes.clear();
        self.detached_input_lines.clear();
        self.detached_list_boxes.clear();
        self.detached_outlines.clear();
        self.detached_radio_buttons.clear();
        self.detached_memos.clear();
        self.detached_text_viewers.clear();
        self.check_box_cells.clear();
        self.input_line_states.clear();
        self.list_box_states.clear();
        self.outline_states.clear();
        self.radio_button_states.clear();
        self.radio_group_members.clear();
        self.button_states.clear();
        self.static_text_texts.clear();
        self.memo_texts.clear();
        self.text_viewer_texts.clear();
        self.child_parents.clear();
        self.button_clicks.clear();
        self.mouse_hit_targets.clear();
        self.pending_mouse_state_toggles.clear();
        self.desktop_windows.clear();
        self.menu_bars.clear();
        self.status_lines.clear();
        self.attached_menu_bar = None;
        self.attached_status_line = None;
        self.file_dialog_result = None;
        self.dialog_result = None;
    }

    /// Returns `true` after [`Self::open`].
    #[must_use]
    pub fn is_open(&self) -> bool {
        self.session_open
    }

    /// Opens a Turbo Vision session and returns the application handle token.
    pub fn open(&mut self) -> u32 {
        self.reset();
        self.session_open = true;
        let handle = self.registry.allocate(0, ViewKind::Application);
        self.app_handle = Some(handle);
        handle
    }

    /// Application handle for the active session, if any.
    #[must_use]
    #[cfg(test)]
    pub fn app_handle(&self) -> Option<u32> {
        self.app_handle
    }

    /// Inserts a root widget and returns its FPAS handle.
    pub fn insert_root(&mut self, root: TuiRoot, kind: ViewKind) -> u32 {
        let handle = self.registry.allocate(0, kind);
        self.roots.insert(handle, root);
        handle
    }

    /// Mutable access to a root widget by handle.
    pub fn root_mut(&mut self, handle: u32) -> Option<&mut TuiRoot> {
        self.roots.get_mut(&handle)
    }

    /// Removes a root widget (for modal exec).
    pub fn take_root(&mut self, handle: u32) -> Option<TuiRoot> {
        self.roots.remove(&handle)
    }
}

mod chrome;
mod controls;
mod headless_state;
mod selection;
mod text;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn open_allocates_application_handle() {
        let mut session = TurboVisionSession::default();
        let h = session.open();
        assert!(session.is_open());
        assert_eq!(session.app_handle(), Some(h));
        assert_eq!(session.registry.len(), 1);
    }

    #[test]
    fn reset_clears_roots() {
        let mut session = TurboVisionSession::default();
        session.open();
        let bounds = turbo_vision::core::geometry::Rect::new(0, 0, 10, 5);
        session.insert_root(
            TuiRoot::ModalDialog(Dialog::new_modal(bounds, "T")),
            ViewKind::Dialog,
        );
        session.reset();
        assert!(!session.is_open());
        assert!(session.roots.is_empty());
    }
}
