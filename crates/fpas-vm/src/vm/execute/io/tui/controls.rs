//! Turbo Vision control construction and parent attachment bridge.
//!
//! **Documentation:** `docs/pascal/std/tui/app/vm-bridge.md`

use super::handles::{TurboVisionChildHandle, TurboVisionParentHandle};
use super::tv_geometry::unknown_handle_error;
use crate::vm::Worker;
use crate::vm::diagnostics::{VmError, runtime_error};
use crate::vm::shared::{
    TurboVisionCheckBox, TurboVisionInputLine, TurboVisionListBox, TurboVisionObject,
    TurboVisionRadioButton, TurboVisionStaticText,
};
use fpas_bytecode::SourceLocation;
use fpas_bytecode::Value;
use fpas_diagnostics::codes::RUNTIME_INTRINSIC_STACK_STATE_ERROR;
use std::cell::RefCell;
use std::rc::Rc;
use turbo_vision::views::{
    button::Button, checkbox::CheckBox, input_line::InputLine, listbox::ListBox,
    radiobutton::RadioButton, static_text::StaticText,
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
                TurboVisionObject::Button(crate::vm::shared::TurboVisionButton {
                    bounds,
                    text,
                    command_id,
                    attached: false,
                }),
            );
            handle
        });
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
        self.push(Self::turbo_vision_static_text_record(handle))
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

        let _input_line = InputLine::new(bounds, max_length, Rc::new(RefCell::new(text.clone())));
        let bounds = super::tv_geometry::state_rect(bounds);
        let handle = self.with_tui(|tui| {
            let handle = tui.turbo_vision.next_handle;
            tui.turbo_vision.next_handle = handle.saturating_add(1).max(1);
            tui.turbo_vision.objects.insert(
                handle,
                TurboVisionObject::InputLine(TurboVisionInputLine {
                    bounds,
                    text,
                    max_length,
                    attached: false,
                }),
            );
            handle
        });
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
                    items,
                    command_id,
                    attached: false,
                }),
            );
            handle
        });
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
                    checked,
                    attached: false,
                }),
            );
            handle
        });
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
                    selected,
                    attached: false,
                }),
            );
            handle
        });
        self.push(Self::turbo_vision_radio_button_record(handle))
    }

    pub(super) fn turbo_vision_add_child(&mut self, line: SourceLocation) -> Result<(), VmError> {
        let child = self.pop_turbo_vision_child_handle(line)?;
        let parent = self.pop_turbo_vision_parent_handle(line)?;
        self.pop_tui_application(line)?;

        self.with_tui(|tui| {
            let child_handle = child.handle();
            let child_label = child.label();
            let attached = match tui.turbo_vision.objects.get(&child_handle) {
                Some(TurboVisionObject::Button(button))
                    if matches!(child, TurboVisionChildHandle::Button(_)) =>
                {
                    button.attached
                }
                Some(TurboVisionObject::StaticText(static_text))
                    if matches!(child, TurboVisionChildHandle::StaticText(_)) =>
                {
                    static_text.attached
                }
                Some(TurboVisionObject::InputLine(input_line))
                    if matches!(child, TurboVisionChildHandle::InputLine(_)) =>
                {
                    input_line.attached
                }
                Some(TurboVisionObject::ListBox(list_box))
                    if matches!(child, TurboVisionChildHandle::ListBox(_)) =>
                {
                    list_box.attached
                }
                Some(TurboVisionObject::CheckBox(check_box))
                    if matches!(child, TurboVisionChildHandle::CheckBox(_)) =>
                {
                    check_box.attached
                }
                Some(TurboVisionObject::RadioButton(radio_button))
                    if matches!(child, TurboVisionChildHandle::RadioButton(_)) =>
                {
                    radio_button.attached
                }
                _ => return Err(unknown_handle_error(child_label, child_handle, line)),
            };

            if attached {
                return Err(runtime_error(
                    RUNTIME_INTRINSIC_STACK_STATE_ERROR,
                    format!("{child_label} handle {child_handle} is already attached"),
                    "Only add a Turbo Vision child to one parent.",
                    line,
                ));
            }

            match parent {
                TurboVisionParentHandle::Dialog(dialog_handle) => {
                    let Some(TurboVisionObject::Dialog(dialog)) =
                        tui.turbo_vision.objects.get_mut(&dialog_handle)
                    else {
                        return Err(unknown_handle_error("Dialog", dialog_handle, line));
                    };
                    dialog.children.push(child_handle);
                }
                TurboVisionParentHandle::Window(window_handle) => {
                    let Some(TurboVisionObject::Window(window)) =
                        tui.turbo_vision.objects.get_mut(&window_handle)
                    else {
                        return Err(unknown_handle_error("Window", window_handle, line));
                    };
                    window.children.push(child_handle);
                }
            }

            match tui.turbo_vision.objects.get_mut(&child_handle) {
                Some(TurboVisionObject::Button(button)) => button.attached = true,
                Some(TurboVisionObject::StaticText(static_text)) => static_text.attached = true,
                Some(TurboVisionObject::InputLine(input_line)) => input_line.attached = true,
                Some(TurboVisionObject::ListBox(list_box)) => list_box.attached = true,
                Some(TurboVisionObject::CheckBox(check_box)) => check_box.attached = true,
                Some(TurboVisionObject::RadioButton(radio_button)) => radio_button.attached = true,
                _ => {}
            }
            Ok(())
        })?;
        Ok(())
    }

    fn pop_turbo_vision_string_array(
        &mut self,
        label: &'static str,
        line: SourceLocation,
    ) -> Result<Vec<String>, VmError> {
        match self.pop(line)? {
            Value::Array(values) => values
                .into_iter()
                .enumerate()
                .map(|(index, value)| match value {
                    Value::Str(text) => Ok(text),
                    other => Err(runtime_error(
                        RUNTIME_INTRINSIC_STACK_STATE_ERROR,
                        format!("{label}[{index}] must be string, got {}", other.type_name()),
                        "Pass an array of string values.",
                        line,
                    )),
                })
                .collect(),
            other => Err(runtime_error(
                RUNTIME_INTRINSIC_STACK_STATE_ERROR,
                format!("{label} must be array of string, got {}", other.type_name()),
                "Pass a string array, for example `['one', 'two']`.",
                line,
            )),
        }
    }
}

impl TurboVisionChildHandle {
    fn handle(&self) -> u32 {
        match self {
            Self::Button(handle)
            | Self::StaticText(handle)
            | Self::InputLine(handle)
            | Self::ListBox(handle)
            | Self::CheckBox(handle)
            | Self::RadioButton(handle) => *handle,
        }
    }

    fn label(&self) -> &'static str {
        match self {
            Self::Button(_) => "Button",
            Self::StaticText(_) => "StaticText",
            Self::InputLine(_) => "InputLine",
            Self::ListBox(_) => "ListBox",
            Self::CheckBox(_) => "CheckBox",
            Self::RadioButton(_) => "RadioButton",
        }
    }
}
