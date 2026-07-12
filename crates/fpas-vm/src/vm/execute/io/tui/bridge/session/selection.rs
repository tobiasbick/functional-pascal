//! List box, outline, and radio button session state.

use super::*;

impl TurboVisionSession {
    pub fn insert_detached_list_box(
        &mut self,
        list_box: Box<dyn View>,
        items: Vec<String>,
        selection_cell: TurboVisionListSelectionCell,
    ) -> u32 {
        let handle = self.registry.allocate(0, ViewKind::ListBox);
        self.list_box_states.insert(
            handle,
            TuiListBoxState {
                items,
                selection_cell,
            },
        );
        self.detached_list_boxes
            .insert(handle, DetachedListBox { list_box });
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
    pub fn list_box_state_mut(&mut self, handle: u32) -> Option<&mut TuiListBoxState> {
        self.list_box_states.get_mut(&handle)
    }

    /// Inserts a detached outline and returns its FPAS handle.
    pub fn insert_detached_outline(
        &mut self,
        outline: Box<dyn View>,
        roots: Vec<TurboVisionOutlineNode>,
        selection_cell: TurboVisionListSelectionCell,
    ) -> u32 {
        let handle = self.registry.allocate(0, ViewKind::Outline);
        self.outline_states.insert(
            handle,
            TuiOutlineState {
                roots,
                selection_cell,
            },
        );
        self.detached_outlines
            .insert(handle, DetachedOutline { outline });
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
    pub fn outline_state(&self, handle: u32) -> Option<&TuiOutlineState> {
        self.outline_states.get(&handle)
    }

    /// Mutable access to outline host state.
    pub fn outline_state_mut(&mut self, handle: u32) -> Option<&mut TuiOutlineState> {
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
            TuiRadioButtonState {
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
    pub fn radio_button_state(&self, handle: u32) -> Option<&TuiRadioButtonState> {
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
}
