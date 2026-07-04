//! Turbo Vision control construction bridge.
//!
//! **Documentation:** `docs/pascal/std/tui/app/vm-bridge.md`

use super::controls::initial_list_selection;
use crate::vm::Worker;
use crate::vm::diagnostics::{VmError, runtime_error};
use crate::vm::shared::{
    TurboVisionButton, TurboVisionCheckBox, TurboVisionInputLine, TurboVisionListBox,
    TurboVisionMemo, TurboVisionObject, TurboVisionRadioButton, TurboVisionStaticText,
    TurboVisionTextViewer,
};
use crate::vm::turbo_vision_bool_cell::TurboVisionBoolCell;
use crate::vm::turbo_vision_input_text_cell::TurboVisionInputTextCell;
use crate::vm::turbo_vision_list_selection_cell::TurboVisionListSelectionCell;
use fpas_bytecode::SourceLocation;
use fpas_diagnostics::codes::RUNTIME_INTRINSIC_STACK_STATE_ERROR;
use turbo_vision::views::{
    button::Button, checkbox::CheckBox, input_line::InputLine, listbox::ListBox, memo::Memo,
    radiobutton::RadioButton, static_text::StaticText, text_viewer::TextViewer,
};

impl Worker {
    pub(super) fn turbo_vision_create_button(
        &mut self,
        line: SourceLocation,
    ) -> Result<(), VmError> {
        let command_id = self.pop_int(line)?;
        let command_id = u16::try_from(command_id).map_err(|_| {
            runtime_error(
                RUNTIME_INTRINSIC_STACK_STATE_ERROR,
                "Button command id is outside the Turbo Vision u16 range",
                "Use a command id from 0 to 65535.",
                line,
            )
        })?;
        let text = self.pop_turbo_vision_string("Button text", line)?;
        let bounds = self.pop_turbo_vision_rect(line)?;
        self.pop_tui_application(line)?;

        let _button = Button::new(bounds, &text, command_id, false);
        let bounds = super::tv_geometry::state_rect(bounds);
        let handle = self.with_tui(|tui| {
            let handle = tui.turbo_vision.next_handle;
            tui.turbo_vision.next_handle = handle.saturating_add(1).max(1);
            tui.turbo_vision.objects.insert(
                handle,
                TurboVisionObject::Button(TurboVisionButton {
                    bounds,
                    text,
                    command_id,
                    attached: false,
                }),
            );
            handle
        });
        self.mark_turbo_vision_tree_dirty();
        self.push(Self::turbo_vision_button_record(handle))
    }

    pub(super) fn turbo_vision_create_static_text(
        &mut self,
        line: SourceLocation,
    ) -> Result<(), VmError> {
        let text = self.pop_turbo_vision_string("StaticText text", line)?;
        let bounds = self.pop_turbo_vision_rect(line)?;
        self.pop_tui_application(line)?;

        let _static_text = StaticText::new(bounds, &text);
        let bounds = super::tv_geometry::state_rect(bounds);
        let handle = self.with_tui(|tui| {
            let handle = tui.turbo_vision.next_handle;
            tui.turbo_vision.next_handle = handle.saturating_add(1).max(1);
            tui.turbo_vision.objects.insert(
                handle,
                TurboVisionObject::StaticText(TurboVisionStaticText {
                    bounds,
                    text,
                    attached: false,
                }),
            );
            handle
        });
        self.mark_turbo_vision_tree_dirty();
        self.push(Self::turbo_vision_static_text_record(handle))
    }

    pub(super) fn turbo_vision_create_memo(&mut self, line: SourceLocation) -> Result<(), VmError> {
        let text = self.pop_turbo_vision_string("Memo text", line)?;
        let bounds = self.pop_turbo_vision_rect(line)?;
        self.pop_tui_application(line)?;

        let _memo = Memo::new(bounds);
        let bounds = super::tv_geometry::state_rect(bounds);
        let handle = self.with_tui(|tui| {
            let handle = tui.turbo_vision.next_handle;
            tui.turbo_vision.next_handle = handle.saturating_add(1).max(1);
            tui.turbo_vision.objects.insert(
                handle,
                TurboVisionObject::Memo(TurboVisionMemo {
                    bounds,
                    text,
                    attached: false,
                }),
            );
            handle
        });
        self.mark_turbo_vision_tree_dirty();
        self.push(Self::turbo_vision_memo_record(handle))
    }

    pub(super) fn turbo_vision_create_text_viewer(
        &mut self,
        line: SourceLocation,
    ) -> Result<(), VmError> {
        let text = self.pop_turbo_vision_string("TextViewer text", line)?;
        let bounds = self.pop_turbo_vision_rect(line)?;
        self.pop_tui_application(line)?;

        let _text_viewer = TextViewer::new(bounds);
        let bounds = super::tv_geometry::state_rect(bounds);
        let handle = self.with_tui(|tui| {
            let handle = tui.turbo_vision.next_handle;
            tui.turbo_vision.next_handle = handle.saturating_add(1).max(1);
            tui.turbo_vision.objects.insert(
                handle,
                TurboVisionObject::TextViewer(TurboVisionTextViewer {
                    bounds,
                    text,
                    attached: false,
                }),
            );
            handle
        });
        self.mark_turbo_vision_tree_dirty();
        self.push(Self::turbo_vision_text_viewer_record(handle))
    }

    pub(super) fn turbo_vision_create_input_line(
        &mut self,
        line: SourceLocation,
    ) -> Result<(), VmError> {
        let max_length = self.pop_int(line)?;
        let max_length = usize::try_from(max_length).map_err(|_| {
            runtime_error(
                RUNTIME_INTRINSIC_STACK_STATE_ERROR,
                "InputLine MaxLength must be non-negative",
                "Use a MaxLength value from 0 to the platform usize maximum.",
                line,
            )
        })?;
        let text = self.pop_turbo_vision_string("InputLine text", line)?;
        if text.len() > max_length {
            return Err(runtime_error(
                RUNTIME_INTRINSIC_STACK_STATE_ERROR,
                format!(
                    "InputLine text length {} exceeds MaxLength {max_length}",
                    text.len()
                ),
                "Pass a shorter initial text or a larger MaxLength.",
                line,
            ));
        }
        let bounds = self.pop_turbo_vision_rect(line)?;
        self.pop_tui_application(line)?;

        let text_cell = TurboVisionInputTextCell::new(text.clone());
        let _input_line = InputLine::new(bounds, max_length, text_cell.view_binding());
        let bounds = super::tv_geometry::state_rect(bounds);
        let handle = self.with_tui(|tui| {
            let handle = tui.turbo_vision.next_handle;
            tui.turbo_vision.next_handle = handle.saturating_add(1).max(1);
            tui.turbo_vision.objects.insert(
                handle,
                TurboVisionObject::InputLine(TurboVisionInputLine {
                    bounds,
                    max_length,
                    text_cell,
                    attached: false,
                }),
            );
            handle
        });
        self.mark_turbo_vision_tree_dirty();
        self.push(Self::turbo_vision_input_line_record(handle))
    }

    pub(super) fn turbo_vision_create_list_box(
        &mut self,
        line: SourceLocation,
    ) -> Result<(), VmError> {
        let command_id = self.pop_int(line)?;
        let command_id = u16::try_from(command_id).map_err(|_| {
            runtime_error(
                RUNTIME_INTRINSIC_STACK_STATE_ERROR,
                "ListBox command id is outside the Turbo Vision u16 range",
                "Use a command id from 0 to 65535.",
                line,
            )
        })?;
        let items = self.pop_turbo_vision_string_array("ListBox items", line)?;
        let bounds = self.pop_turbo_vision_rect(line)?;
        self.pop_tui_application(line)?;

        let mut list_box = ListBox::new(bounds, command_id);
        list_box.set_items(items.clone());
        let bounds = super::tv_geometry::state_rect(bounds);
        let handle = self.with_tui(|tui| {
            let handle = tui.turbo_vision.next_handle;
            tui.turbo_vision.next_handle = handle.saturating_add(1).max(1);
            tui.turbo_vision.objects.insert(
                handle,
                TurboVisionObject::ListBox(TurboVisionListBox {
                    bounds,
                    selection_cell: TurboVisionListSelectionCell::new(initial_list_selection(
                        &items,
                    )),
                    items,
                    command_id,
                    attached: false,
                }),
            );
            handle
        });
        self.mark_turbo_vision_tree_dirty();
        self.push(Self::turbo_vision_list_box_record(handle))
    }

    pub(super) fn turbo_vision_create_check_box(
        &mut self,
        line: SourceLocation,
    ) -> Result<(), VmError> {
        let checked = self.pop_bool(line)?;
        let text = self.pop_turbo_vision_string("CheckBox text", line)?;
        let bounds = self.pop_turbo_vision_rect(line)?;
        self.pop_tui_application(line)?;

        let mut check_box = CheckBox::new(bounds, &text);
        check_box.set_checked(checked);
        let bounds = super::tv_geometry::state_rect(bounds);
        let handle = self.with_tui(|tui| {
            let handle = tui.turbo_vision.next_handle;
            tui.turbo_vision.next_handle = handle.saturating_add(1).max(1);
            tui.turbo_vision.objects.insert(
                handle,
                TurboVisionObject::CheckBox(TurboVisionCheckBox {
                    bounds,
                    text,
                    checked_cell: TurboVisionBoolCell::new(checked),
                    attached: false,
                }),
            );
            handle
        });
        self.mark_turbo_vision_tree_dirty();
        self.push(Self::turbo_vision_check_box_record(handle))
    }

    pub(super) fn turbo_vision_create_radio_button(
        &mut self,
        line: SourceLocation,
    ) -> Result<(), VmError> {
        let selected = self.pop_bool(line)?;
        let group_id = self.pop_int(line)?;
        let group_id = u16::try_from(group_id).map_err(|_| {
            runtime_error(
                RUNTIME_INTRINSIC_STACK_STATE_ERROR,
                "RadioButton group id is outside the Turbo Vision u16 range",
                "Use a group id from 0 to 65535.",
                line,
            )
        })?;
        let text = self.pop_turbo_vision_string("RadioButton text", line)?;
        let bounds = self.pop_turbo_vision_rect(line)?;
        self.pop_tui_application(line)?;

        let mut radio_button = RadioButton::new(bounds, &text, group_id);
        if selected {
            radio_button.select();
        }
        let bounds = super::tv_geometry::state_rect(bounds);
        let handle = self.with_tui(|tui| {
            let handle = tui.turbo_vision.next_handle;
            tui.turbo_vision.next_handle = handle.saturating_add(1).max(1);
            tui.turbo_vision.objects.insert(
                handle,
                TurboVisionObject::RadioButton(TurboVisionRadioButton {
                    bounds,
                    text,
                    group_id,
                    selected_cell: TurboVisionBoolCell::new(selected),
                    attached: false,
                }),
            );
            handle
        });
        self.mark_turbo_vision_tree_dirty();
        self.push(Self::turbo_vision_radio_button_record(handle))
    }
}
