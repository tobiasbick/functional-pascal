//! Detached and attached button, text, check box, and input-line state.

use super::*;

impl TurboVisionSession {
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
            TuiButtonState {
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
    pub fn button_state(&self, handle: u32) -> Option<&TuiButtonState> {
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
    /// Read host-side static text (unit tests).
    #[cfg(test)]
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
            TuiInputLineState {
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
}
