//! Try-2 TUI session state: view registry and owned widget roots.
//!
//! **Documentation:** `docs/refactor-tui-try-2/target-architecture.md`

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
pub(crate) struct Try2ButtonState {
    pub command: u16,
    pub is_default: bool,
    pub text: String,
}

/// Check box constructed by `CheckBox.New` and not yet attached to a parent.
pub(crate) struct DetachedCheckBox {
    pub check_box: Box<dyn View>,
    pub local_bounds: Rect,
    pub checked_cell: TurboVisionBoolCell,
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
    pub local_bounds: Rect,
}

/// Outline constructed by `Outline.New` and not yet attached to a parent.
pub(crate) struct DetachedOutline {
    pub outline: Box<dyn View>,
    pub local_bounds: Rect,
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

/// Host-side radio button state retained after attach.
#[derive(Clone)]
pub(crate) struct Try2RadioButtonState {
    pub bounds: Rect,
    pub text: String,
    pub group_id: u16,
    pub selected_cell: TurboVisionBoolCell,
}

/// Host-side list box state retained after attach.
pub(crate) struct Try2ListBoxState {
    pub items: Vec<String>,
    pub command_id: u16,
    pub selection_cell: TurboVisionListSelectionCell,
}

/// Host-side outline state retained after attach.
pub(crate) struct Try2OutlineState {
    pub roots: Vec<TurboVisionOutlineNode>,
    pub selection_cell: TurboVisionListSelectionCell,
}

/// Host-side input line state retained after attach.
pub(crate) struct Try2InputLineState {
    pub text_cell: TurboVisionInputTextCell,
    pub max_length: usize,
    pub view_binding: Option<Rc<RefCell<String>>>,
}

/// Menu bar data owned by try-2 until attached via `Application.SetMenuBar`.
pub(crate) struct Try2MenuBarState {
    pub bounds: TurboVisionRect,
    pub menus: Vec<TurboVisionMenu>,
}

/// Status line data owned by try-2 until attached via `Application.SetStatusLine`.
pub(crate) struct Try2StatusLineState {
    pub bounds: TurboVisionRect,
    pub items: Vec<TurboVisionStatusItem>,
}

/// Owned top-level widget waiting for modal exec or desktop attach.
pub(crate) enum Try2Root {
    ModalDialog(Box<Dialog>),
    Window(Box<Window>),
}

/// Rust-owned try-2 session (coexists with try-1 `TurboVisionState` until phase 7).
#[derive(Default)]
pub(crate) struct Try2Session {
    pub registry: ViewRegistry,
    session_open: bool,
    app_handle: Option<u32>,
    roots: HashMap<u32, Try2Root>,
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
    /// Host-side check box state keyed by try-2 handle.
    check_box_cells: HashMap<u32, TurboVisionBoolCell>,
    /// Host-side input line state keyed by try-2 handle.
    input_line_states: HashMap<u32, Try2InputLineState>,
    /// Host-side list box state keyed by try-2 handle.
    list_box_states: HashMap<u32, Try2ListBoxState>,
    /// Host-side outline state keyed by try-2 handle.
    outline_states: HashMap<u32, Try2OutlineState>,
    /// Host-side radio button state keyed by try-2 handle.
    radio_button_states: HashMap<u32, Try2RadioButtonState>,
    /// Radio group members keyed by FPAS `GroupId`.
    radio_group_members: HashMap<u16, Vec<u32>>,
    /// Host-side button state keyed by try-2 handle.
    button_states: HashMap<u32, Try2ButtonState>,
    /// Host-side static text keyed by try-2 handle.
    static_text_texts: HashMap<u32, String>,
    /// Host-side memo text keyed by try-2 handle.
    memo_texts: HashMap<u32, String>,
    /// Host-side text viewer text keyed by try-2 handle.
    text_viewer_texts: HashMap<u32, String>,
    /// Child handle to parent dialog/window handle after attach.
    child_parents: HashMap<u32, u32>,
    /// Headless test click targets keyed by try-2 button handle.
    button_clicks: HashMap<u32, Point>,
    /// Screen-space mouse hit targets for check boxes and radio buttons.
    mouse_hit_targets: Vec<(Rect, Point)>,
    /// Window handles attached to the upstream desktop via `Desktop.Add`.
    desktop_windows: HashSet<u32>,
    menu_bars: HashMap<u32, Try2MenuBarState>,
    status_lines: HashMap<u32, Try2StatusLineState>,
    attached_menu_bar: Option<u32>,
    attached_status_line: Option<u32>,
    /// Headless file dialog result queued by the interim test helper.
    file_dialog_result: Option<Option<String>>,
}

impl Try2Session {
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
        self.desktop_windows.clear();
        self.menu_bars.clear();
        self.status_lines.clear();
        self.attached_menu_bar = None;
        self.attached_status_line = None;
        self.file_dialog_result = None;
    }

    /// Returns `true` after [`Self::open`].
    #[must_use]
    pub fn is_open(&self) -> bool {
        self.session_open
    }

    /// Opens a try-2 session and returns the application handle token.
    pub fn open(&mut self) -> u32 {
        self.reset();
        self.session_open = true;
        let handle = self.registry.allocate(0, ViewKind::Application);
        self.app_handle = Some(handle);
        handle
    }

    /// Application handle for the active session, if any.
    #[must_use]
    pub fn app_handle(&self) -> Option<u32> {
        self.app_handle
    }

    /// Inserts a root widget and returns its FPAS handle.
    pub fn insert_root(&mut self, root: Try2Root, kind: ViewKind) -> u32 {
        let handle = self.registry.allocate(0, kind);
        self.roots.insert(handle, root);
        handle
    }

    /// Mutable access to a root widget by handle.
    pub fn root_mut(&mut self, handle: u32) -> Option<&mut Try2Root> {
        self.roots.get_mut(&handle)
    }

    /// Removes a root widget (for modal exec).
    pub fn take_root(&mut self, handle: u32) -> Option<Try2Root> {
        self.roots.remove(&handle)
    }

    /// Inserts a detached button and returns its FPAS handle.
    pub fn insert_detached_button(
        &mut self,
        button: Box<dyn View>,
        local_bounds: Rect,
        command: u16,
        is_default: bool,
        text: String,
    ) -> u32 {
        let handle = self.registry.allocate(0, ViewKind::Button);
        self.button_states.insert(
            handle,
            Try2ButtonState {
                command,
                is_default,
                text,
            },
        );
        self.detached_buttons.insert(
            handle,
            DetachedButton {
                button,
                local_bounds,
            },
        );
        handle
    }

    /// Replaces a detached button view after `Button.SetText`.
    pub fn replace_detached_button(&mut self, handle: u32, button: Box<dyn View>) {
        if let Some(detached) = self.detached_buttons.get_mut(&handle) {
            detached.button = button;
        }
    }

    /// Returns detached button bounds when still awaiting attach.
    #[must_use]
    pub fn detached_button_bounds(&self, handle: u32) -> Option<Rect> {
        self.detached_buttons
            .get(&handle)
            .map(|detached| detached.local_bounds)
    }

    /// Returns host-side button state.
    #[must_use]
    pub fn button_state(&self, handle: u32) -> Option<&Try2ButtonState> {
        self.button_states.get(&handle)
    }

    /// Updates host-side button text.
    pub fn set_button_text(&mut self, handle: u32, text: String) {
        if let Some(state) = self.button_states.get_mut(&handle) {
            state.text = text;
        }
    }

    /// Removes a detached button for parent attach.
    pub fn take_detached_button(&mut self, handle: u32) -> Option<DetachedButton> {
        self.detached_buttons.remove(&handle)
    }

    /// Inserts a detached static text and returns its FPAS handle.
    pub fn insert_detached_static_text(
        &mut self,
        static_text: Box<dyn View>,
        local_bounds: Rect,
        text: String,
    ) -> u32 {
        let handle = self.registry.allocate(0, ViewKind::StaticText);
        self.static_text_texts.insert(handle, text);
        self.detached_static_texts.insert(
            handle,
            DetachedStaticText {
                static_text,
                local_bounds,
            },
        );
        handle
    }

    /// Replaces a detached static text view after `StaticText.SetText`.
    pub fn replace_detached_static_text(&mut self, handle: u32, static_text: Box<dyn View>) {
        if let Some(detached) = self.detached_static_texts.get_mut(&handle) {
            detached.static_text = static_text;
        }
    }

    /// Returns detached static text bounds when still awaiting attach.
    #[must_use]
    pub fn detached_static_text_bounds(&self, handle: u32) -> Option<Rect> {
        self.detached_static_texts
            .get(&handle)
            .map(|detached| detached.local_bounds)
    }

    /// Returns host-side static text.
    #[must_use]
    pub fn static_text_text(&self, handle: u32) -> Option<&str> {
        self.static_text_texts.get(&handle).map(String::as_str)
    }

    /// Updates host-side static text.
    pub fn set_static_text_text(&mut self, handle: u32, text: String) {
        self.static_text_texts.insert(handle, text);
    }

    /// Removes a detached static text for parent attach.
    pub fn take_detached_static_text(&mut self, handle: u32) -> Option<DetachedStaticText> {
        self.detached_static_texts.remove(&handle)
    }

    /// Inserts a detached check box and returns its FPAS handle.
    pub fn insert_detached_check_box(
        &mut self,
        check_box: Box<dyn View>,
        local_bounds: Rect,
        checked_cell: TurboVisionBoolCell,
    ) -> u32 {
        let handle = self.registry.allocate(0, ViewKind::CheckBox);
        self.check_box_cells.insert(handle, checked_cell.clone());
        self.detached_check_boxes.insert(
            handle,
            DetachedCheckBox {
                check_box,
                local_bounds,
                checked_cell,
            },
        );
        handle
    }

    /// Removes a detached check box for parent attach.
    pub fn take_detached_check_box(&mut self, handle: u32) -> Option<DetachedCheckBox> {
        self.detached_check_boxes.remove(&handle)
    }

    /// Inserts a detached input line and returns its FPAS handle.
    pub fn insert_detached_input_line(
        &mut self,
        local_bounds: Rect,
        text_cell: TurboVisionInputTextCell,
        max_length: usize,
    ) -> u32 {
        let handle = self.registry.allocate(0, ViewKind::InputLine);
        self.input_line_states.insert(
            handle,
            Try2InputLineState {
                text_cell: text_cell.clone(),
                max_length,
                view_binding: None,
            },
        );
        self.detached_input_lines.insert(
            handle,
            DetachedInputLine {
                local_bounds,
                text_cell,
                max_length,
            },
        );
        handle
    }

    /// Removes a detached input line for parent attach.
    pub fn take_detached_input_line(&mut self, handle: u32) -> Option<DetachedInputLine> {
        self.detached_input_lines.remove(&handle)
    }

    /// Returns the shared checked cell for a check box handle.
    #[must_use]
    pub fn check_box_cell(&self, handle: u32) -> Option<&TurboVisionBoolCell> {
        self.check_box_cells.get(&handle)
    }

    /// Returns the shared input line text cell for a handle.
    #[must_use]
    pub fn input_line_cell(&self, handle: u32) -> Option<&TurboVisionInputTextCell> {
        self.input_line_states
            .get(&handle)
            .map(|state| &state.text_cell)
    }

    /// Returns the configured max length for an input line handle.
    #[must_use]
    pub fn input_line_max_length(&self, handle: u32) -> Option<usize> {
        self.input_line_states
            .get(&handle)
            .map(|state| state.max_length)
    }

    /// Stores the live view binding for an attached input line.
    pub fn set_input_line_binding(&mut self, handle: u32, binding: Rc<RefCell<String>>) {
        if let Some(state) = self.input_line_states.get_mut(&handle) {
            state.view_binding = Some(binding);
        }
    }

    /// Returns the live view binding for an attached input line.
    #[must_use]
    pub fn input_line_binding(&self, handle: u32) -> Option<Rc<RefCell<String>>> {
        self.input_line_states
            .get(&handle)
            .and_then(|state| state.view_binding.clone())
    }

    /// Copies edited view text into the host input line cell when a binding exists.
    pub fn commit_input_line_text(&mut self, handle: u32) {
        let Some(binding) = self
            .input_line_states
            .get(&handle)
            .and_then(|state| state.view_binding.clone())
        else {
            return;
        };
        if let Some(state) = self.input_line_states.get_mut(&handle) {
            state.text_cell.commit_view_binding(&binding);
        }
    }

    /// Records the parent dialog or window for an attached child handle.
    pub fn set_child_parent(&mut self, child_handle: u32, parent_handle: u32) {
        self.child_parents.insert(child_handle, parent_handle);
    }

    /// Parent dialog or window handle for an attached child, if any.
    #[must_use]
    pub fn child_parent(&self, child_handle: u32) -> Option<u32> {
        self.child_parents.get(&child_handle).copied()
    }

    /// Inserts a detached list box and returns its FPAS handle.
    pub fn insert_detached_list_box(
        &mut self,
        list_box: Box<dyn View>,
        local_bounds: Rect,
        items: Vec<String>,
        command_id: u16,
        selection_cell: TurboVisionListSelectionCell,
    ) -> u32 {
        let handle = self.registry.allocate(0, ViewKind::ListBox);
        self.list_box_states.insert(
            handle,
            Try2ListBoxState {
                items,
                command_id,
                selection_cell,
            },
        );
        self.detached_list_boxes.insert(
            handle,
            DetachedListBox {
                list_box,
                local_bounds,
            },
        );
        handle
    }

    /// Removes a detached list box for parent attach.
    pub fn take_detached_list_box(&mut self, handle: u32) -> Option<DetachedListBox> {
        self.detached_list_boxes.remove(&handle)
    }

    /// Returns the shared list box selection cell for a handle.
    #[must_use]
    pub fn list_box_selection_cell(&self, handle: u32) -> Option<&TurboVisionListSelectionCell> {
        self.list_box_states
            .get(&handle)
            .map(|state| &state.selection_cell)
    }

    /// Mutable access to list box host state.
    pub fn list_box_state_mut(&mut self, handle: u32) -> Option<&mut Try2ListBoxState> {
        self.list_box_states.get_mut(&handle)
    }

    /// Inserts a detached outline and returns its FPAS handle.
    pub fn insert_detached_outline(
        &mut self,
        outline: Box<dyn View>,
        local_bounds: Rect,
        roots: Vec<TurboVisionOutlineNode>,
        selection_cell: TurboVisionListSelectionCell,
    ) -> u32 {
        let handle = self.registry.allocate(0, ViewKind::Outline);
        self.outline_states.insert(
            handle,
            Try2OutlineState {
                roots,
                selection_cell,
            },
        );
        self.detached_outlines.insert(
            handle,
            DetachedOutline {
                outline,
                local_bounds,
            },
        );
        handle
    }

    /// Removes a detached outline for parent attach.
    pub fn take_detached_outline(&mut self, handle: u32) -> Option<DetachedOutline> {
        self.detached_outlines.remove(&handle)
    }

    /// Returns the shared outline selection cell for a handle.
    #[must_use]
    pub fn outline_selection_cell(&self, handle: u32) -> Option<&TurboVisionListSelectionCell> {
        self.outline_states
            .get(&handle)
            .map(|state| &state.selection_cell)
    }

    /// Read-only outline host state.
    #[must_use]
    pub fn outline_state(&self, handle: u32) -> Option<&Try2OutlineState> {
        self.outline_states.get(&handle)
    }

    /// Mutable access to outline host state.
    pub fn outline_state_mut(&mut self, handle: u32) -> Option<&mut Try2OutlineState> {
        self.outline_states.get_mut(&handle)
    }

    /// Registers radio button host state and returns its FPAS handle.
    pub fn insert_radio_button_state(
        &mut self,
        bounds: Rect,
        text: String,
        group_id: u16,
        selected_cell: TurboVisionBoolCell,
    ) -> u32 {
        let handle = self.registry.allocate(0, ViewKind::RadioButton);
        self.radio_button_states.insert(
            handle,
            Try2RadioButtonState {
                bounds,
                text,
                group_id,
                selected_cell,
            },
        );
        self.radio_group_members
            .entry(group_id)
            .or_default()
            .push(handle);
        handle
    }

    /// Inserts a detached radio button view for an existing handle.
    pub fn insert_detached_radio_button(
        &mut self,
        handle: u32,
        radio_button: Box<dyn View>,
        local_bounds: Rect,
    ) {
        self.detached_radio_buttons.insert(
            handle,
            DetachedRadioButton {
                radio_button,
                local_bounds,
            },
        );
    }

    /// Replaces a detached radio button view after group membership changes.
    pub fn replace_detached_radio_button(&mut self, handle: u32, radio_button: Box<dyn View>) {
        if let Some(detached) = self.detached_radio_buttons.get_mut(&handle) {
            detached.radio_button = radio_button;
        }
    }

    /// Removes a detached radio button for parent attach.
    pub fn take_detached_radio_button(&mut self, handle: u32) -> Option<DetachedRadioButton> {
        self.detached_radio_buttons.remove(&handle)
    }

    /// Snapshot of radio button host state.
    #[must_use]
    pub fn radio_button_state(&self, handle: u32) -> Option<&Try2RadioButtonState> {
        self.radio_button_states.get(&handle)
    }

    /// Returns the shared selected cell for a radio button handle.
    #[must_use]
    pub fn radio_button_selected_cell(&self, handle: u32) -> Option<&TurboVisionBoolCell> {
        self.radio_button_states
            .get(&handle)
            .map(|state| &state.selected_cell)
    }

    /// Returns live handles in a radio group.
    #[must_use]
    pub fn radio_group_member_handles(&self, group_id: u16) -> Vec<u32> {
        self.radio_group_members
            .get(&group_id)
            .cloned()
            .unwrap_or_default()
    }

    /// Returns shared selection cells for all members of a radio group.
    #[must_use]
    pub fn radio_group_cells(&self, group_id: u16) -> Vec<TurboVisionBoolCell> {
        self.radio_group_member_handles(group_id)
            .into_iter()
            .filter_map(|handle| {
                self.radio_button_states
                    .get(&handle)
                    .map(|state| state.selected_cell.clone())
            })
            .collect()
    }

    /// Clears selection for every member of a radio group except `keep`.
    pub fn deselect_radio_group_except(&mut self, group_id: u16, keep: Option<u32>) {
        for handle in self.radio_group_member_handles(group_id) {
            if keep == Some(handle) {
                continue;
            }
            if let Some(state) = self.radio_button_states.get(&handle) {
                state.selected_cell.set(false);
            }
        }
    }

    /// Inserts a detached memo and returns its FPAS handle.
    pub fn insert_detached_memo(
        &mut self,
        memo: Box<dyn View>,
        local_bounds: Rect,
        text: String,
    ) -> u32 {
        let handle = self.registry.allocate(0, ViewKind::Memo);
        self.memo_texts.insert(handle, text);
        self.detached_memos
            .insert(handle, DetachedMemo { memo, local_bounds });
        handle
    }

    /// Replaces a detached memo view after `Memo.SetText`.
    pub fn replace_detached_memo(&mut self, handle: u32, memo: Box<dyn View>) {
        if let Some(detached) = self.detached_memos.get_mut(&handle) {
            detached.memo = memo;
        }
    }

    /// Removes a detached memo for parent attach.
    pub fn take_detached_memo(&mut self, handle: u32) -> Option<DetachedMemo> {
        self.detached_memos.remove(&handle)
    }

    /// Returns host-side memo text.
    #[must_use]
    pub fn memo_text(&self, handle: u32) -> Option<&str> {
        self.memo_texts.get(&handle).map(String::as_str)
    }

    /// Updates host-side memo text.
    pub fn set_memo_text(&mut self, handle: u32, text: String) {
        self.memo_texts.insert(handle, text);
    }

    /// Returns detached memo bounds when still awaiting attach.
    #[must_use]
    pub fn detached_memo_bounds(&self, handle: u32) -> Option<Rect> {
        self.detached_memos
            .get(&handle)
            .map(|detached| detached.local_bounds)
    }

    /// Inserts a detached text viewer and returns its FPAS handle.
    pub fn insert_detached_text_viewer(
        &mut self,
        text_viewer: Box<dyn View>,
        local_bounds: Rect,
        text: String,
    ) -> u32 {
        let handle = self.registry.allocate(0, ViewKind::TextViewer);
        self.text_viewer_texts.insert(handle, text);
        self.detached_text_viewers.insert(
            handle,
            DetachedTextViewer {
                text_viewer,
                local_bounds,
            },
        );
        handle
    }

    /// Replaces a detached text viewer after `TextViewer.SetText`.
    pub fn replace_detached_text_viewer(&mut self, handle: u32, text_viewer: Box<dyn View>) {
        if let Some(detached) = self.detached_text_viewers.get_mut(&handle) {
            detached.text_viewer = text_viewer;
        }
    }

    /// Removes a detached text viewer for parent attach.
    pub fn take_detached_text_viewer(&mut self, handle: u32) -> Option<DetachedTextViewer> {
        self.detached_text_viewers.remove(&handle)
    }

    /// Returns host-side text viewer text.
    #[must_use]
    pub fn text_viewer_text(&self, handle: u32) -> Option<&str> {
        self.text_viewer_texts.get(&handle).map(String::as_str)
    }

    /// Updates host-side text viewer text.
    pub fn set_text_viewer_text(&mut self, handle: u32, text: String) {
        self.text_viewer_texts.insert(handle, text);
    }

    /// Returns detached text viewer bounds when still awaiting attach.
    #[must_use]
    pub fn detached_text_viewer_bounds(&self, handle: u32) -> Option<Rect> {
        self.detached_text_viewers
            .get(&handle)
            .map(|detached| detached.local_bounds)
    }

    /// Returns the command id for a menu bar item, if present and not a separator.
    pub fn menu_item_command_id(
        &self,
        handle: u32,
        menu_index: usize,
        item_index: usize,
    ) -> Option<u16> {
        let state = self.menu_bars.get(&handle)?;
        let menu = state.menus.get(menu_index)?;
        let item = menu.items.get(item_index)?;
        if item.command_id == 0 {
            None
        } else {
            Some(item.command_id)
        }
    }

    /// Stores menu bar data for a registry handle.
    pub fn insert_menu_bar(&mut self, handle: u32, state: Try2MenuBarState) {
        self.menu_bars.insert(handle, state);
    }

    /// Replaces menu bar menus (`MenuBar.SetMenus`).
    pub fn set_menu_bar_menus(
        &mut self,
        handle: u32,
        menus: Vec<TurboVisionMenu>,
    ) -> Result<(), ()> {
        let Some(state) = self.menu_bars.get_mut(&handle) else {
            return Err(());
        };
        state.menus = menus;
        Ok(())
    }

    /// Stores status line data for a registry handle.
    pub fn insert_status_line(&mut self, handle: u32, state: Try2StatusLineState) {
        self.status_lines.insert(handle, state);
    }

    /// Replaces status line items (`StatusLine.SetItems`).
    pub fn set_status_line_items(
        &mut self,
        handle: u32,
        items: Vec<TurboVisionStatusItem>,
    ) -> Result<(), ()> {
        let Some(state) = self.status_lines.get_mut(&handle) else {
            return Err(());
        };
        state.items = items;
        Ok(())
    }

    /// Returns the attached menu bar handle, if any.
    #[must_use]
    pub fn attached_menu_bar(&self) -> Option<u32> {
        self.attached_menu_bar
    }

    /// Returns the attached status line handle, if any.
    #[must_use]
    pub fn attached_status_line(&self) -> Option<u32> {
        self.attached_status_line
    }

    /// Marks a menu bar as application chrome.
    pub fn set_attached_menu_bar(&mut self, handle: u32) {
        self.attached_menu_bar = Some(handle);
    }

    /// Marks a status line as application chrome.
    pub fn set_attached_status_line(&mut self, handle: u32) {
        self.attached_status_line = Some(handle);
    }

    /// Snapshot of the attached menu bar, if set.
    #[must_use]
    pub fn attached_menu_bar_snapshot(&self) -> Option<&Try2MenuBarState> {
        self.attached_menu_bar
            .and_then(|handle| self.menu_bars.get(&handle))
    }

    /// Snapshot of the attached status line, if set.
    #[must_use]
    pub fn attached_status_line_snapshot(&self) -> Option<&Try2StatusLineState> {
        self.attached_status_line
            .and_then(|handle| self.status_lines.get(&handle))
    }

    /// Records the screen point used by headless `Application.TestClickButton`.
    pub fn set_button_click_point(&mut self, handle: u32, point: Point) {
        self.button_clicks.insert(handle, point);
    }

    /// Screen point for a try-2 button handle, if registered.
    #[must_use]
    pub fn button_click_point(&self, handle: u32) -> Option<Point> {
        self.button_clicks.get(&handle).copied()
    }

    /// Registers a screen-space hit target for headless `TestClickMouse`.
    pub fn register_mouse_hit_target(&mut self, hit: Rect, click: Point) {
        self.mouse_hit_targets.push((hit, click));
    }

    /// Resolves a queued screen click to a desktop mouse coordinate when possible.
    #[must_use]
    pub fn mouse_point_for_screen(&self, x: i16, y: i16) -> Option<Point> {
        for (hit, click) in &self.mouse_hit_targets {
            if super::view_click::point_in_screen_bounds(*hit, x, y) {
                return Some(*click);
            }
        }
        None
    }

    /// Validates session is open.
    pub fn require_open(&self) -> bool {
        self.session_open
    }

    /// Returns `true` when `handle` was passed to `Desktop.Add`.
    #[must_use]
    pub fn is_on_desktop(&self, handle: u32) -> bool {
        self.desktop_windows.contains(&handle)
    }

    /// Records that a window handle now lives on the upstream desktop.
    pub fn mark_desktop_window(&mut self, handle: u32) {
        self.desktop_windows.insert(handle);
    }

    /// Queues the result consumed by the next headless `Application.RunFileDialog`.
    pub fn set_file_dialog_result(&mut self, result: Option<String>) {
        self.file_dialog_result = Some(result);
    }

    /// Consumes the queued headless file dialog result, if one was set.
    #[must_use]
    pub fn take_file_dialog_result(&mut self) -> Option<Option<String>> {
        self.file_dialog_result.take()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn open_allocates_application_handle() {
        let mut session = Try2Session::default();
        let h = session.open();
        assert!(session.is_open());
        assert_eq!(session.app_handle(), Some(h));
        assert_eq!(session.registry.len(), 1);
    }

    #[test]
    fn reset_clears_roots() {
        let mut session = Try2Session::default();
        session.open();
        let bounds = turbo_vision::core::geometry::Rect::new(0, 0, 10, 5);
        session.insert_root(
            Try2Root::ModalDialog(Dialog::new_modal(bounds, "T")),
            ViewKind::Dialog,
        );
        session.reset();
        assert!(!session.is_open());
        assert!(session.roots.is_empty());
    }
}
