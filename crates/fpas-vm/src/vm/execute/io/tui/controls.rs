//! Turbo Vision child attachment and mutable control state bridge.
//!
//! **Documentation:** `docs/pascal/std/tui/app/vm-bridge.md`

use super::handles::{CheckedControlHandle, TurboVisionChildHandle, TurboVisionParentHandle};
use super::live_patch::LiveDataMutation;
use super::tv_geometry::unknown_handle_error;
use crate::vm::Worker;
use crate::vm::diagnostics::{VmError, runtime_error};
use crate::vm::shared::TurboVisionObject;
use fpas_bytecode::SourceLocation;
use fpas_bytecode::Value;
use fpas_diagnostics::codes::RUNTIME_INTRINSIC_STACK_STATE_ERROR;

impl Worker {
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
                Some(TurboVisionObject::Memo(memo))
                    if matches!(child, TurboVisionChildHandle::Memo(_)) =>
                {
                    memo.attached
                }
                Some(TurboVisionObject::TextViewer(text_viewer))
                    if matches!(child, TurboVisionChildHandle::TextViewer(_)) =>
                {
                    text_viewer.attached
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
                Some(TurboVisionObject::Memo(memo)) => memo.attached = true,
                Some(TurboVisionObject::TextViewer(text_viewer)) => text_viewer.attached = true,
                Some(TurboVisionObject::InputLine(input_line)) => input_line.attached = true,
                Some(TurboVisionObject::ListBox(list_box)) => list_box.attached = true,
                Some(TurboVisionObject::CheckBox(check_box)) => check_box.attached = true,
                Some(TurboVisionObject::RadioButton(radio_button)) => radio_button.attached = true,
                _ => {}
            }
            Ok(())
        })?;
        self.mark_turbo_vision_tree_dirty();
        Ok(())
    }

    /// Replace the text of a text-bearing control handle at runtime.
    ///
    /// **Documentation:** `docs/pascal/std/tui/app/controls.md`
    pub(super) fn turbo_vision_set_text(&mut self, line: SourceLocation) -> Result<(), VmError> {
        let text = self.pop_turbo_vision_string("SetText text", line)?;
        let control = self.pop_turbo_vision_child_handle(line)?;
        self.pop_tui_application(line)?;

        let handle = control.handle();
        let label = control.label();
        let live_patch = self.with_tui(|tui| -> Result<Option<LiveDataMutation>, VmError> {
            match tui.turbo_vision.objects.get_mut(&handle) {
                Some(TurboVisionObject::Memo(memo)) => {
                    memo.text = text;
                    Ok(Some(LiveDataMutation::SetText { handle }))
                }
                Some(TurboVisionObject::TextViewer(text_viewer)) => {
                    text_viewer.text = text;
                    Ok(Some(LiveDataMutation::SetText { handle }))
                }
                Some(TurboVisionObject::InputLine(input_line)) => {
                    if text.len() > input_line.max_length {
                        return Err(runtime_error(
                            RUNTIME_INTRINSIC_STACK_STATE_ERROR,
                            format!(
                                "InputLine text length {} exceeds MaxLength {}",
                                text.len(),
                                input_line.max_length
                            ),
                            "Pass a shorter text or recreate the input line with a larger MaxLength.",
                            line,
                        ));
                    }
                    input_line.text_cell.set(text.clone());
                    self.turbo_vision_sync_input_line_view_binding(handle, &text);
                    Ok(Some(LiveDataMutation::SetText { handle }))
                }
                Some(TurboVisionObject::Button(button)) => {
                    button.text = text;
                    Ok(Some(LiveDataMutation::SetText { handle }))
                }
                Some(TurboVisionObject::StaticText(static_text)) => {
                    static_text.text = text;
                    Ok(Some(LiveDataMutation::SetText { handle }))
                }
                Some(TurboVisionObject::CheckBox(check_box)) => {
                    check_box.text = text;
                    Ok(Some(LiveDataMutation::SetText { handle }))
                }
                Some(TurboVisionObject::RadioButton(radio_button)) => {
                    radio_button.text = text;
                    Ok(Some(LiveDataMutation::SetText { handle }))
                }
                Some(TurboVisionObject::ListBox(_)) => Err(runtime_error(
                    RUNTIME_INTRINSIC_STACK_STATE_ERROR,
                    "Application.SetText does not support ListBox handles",
                    "Recreate the list box with `Application.CreateListBox` to change its items.",
                    line,
                )),
                _ => Err(unknown_handle_error(label, handle, line)),
            }
        })?;

        if let Some(mutation) = live_patch {
            self.turbo_vision_after_data_mutation(mutation);
        } else {
            self.mark_turbo_vision_tree_dirty();
        }
        Ok(())
    }

    /// Replace the checked/selected state of a check box or radio button.
    ///
    /// **Documentation:** `docs/pascal/std/tui/app/controls.md`
    pub(super) fn turbo_vision_set_checked(&mut self, line: SourceLocation) -> Result<(), VmError> {
        let checked = self.pop_bool(line)?;
        let control = self.pop_turbo_vision_checked_control_handle(line)?;
        self.pop_tui_application(line)?;

        self.with_tui(|tui| match control {
            CheckedControlHandle::CheckBox(handle) => {
                let Some(TurboVisionObject::CheckBox(check_box)) =
                    tui.turbo_vision.objects.get_mut(&handle)
                else {
                    return Err(unknown_handle_error("CheckBox", handle, line));
                };
                check_box.checked_cell.set(checked);
                Ok(())
            }
            CheckedControlHandle::RadioButton(handle) => {
                let group_id = match tui.turbo_vision.objects.get(&handle) {
                    Some(TurboVisionObject::RadioButton(radio_button)) => radio_button.group_id,
                    _ => return Err(unknown_handle_error("RadioButton", handle, line)),
                };
                if checked {
                    for object in tui.turbo_vision.objects.values_mut() {
                        let TurboVisionObject::RadioButton(radio_button) = object else {
                            continue;
                        };
                        if radio_button.group_id == group_id {
                            radio_button.selected_cell.set(false);
                        }
                    }
                }
                let Some(TurboVisionObject::RadioButton(radio_button)) =
                    tui.turbo_vision.objects.get_mut(&handle)
                else {
                    return Err(unknown_handle_error("RadioButton", handle, line));
                };
                radio_button.selected_cell.set(checked);
                Ok(())
            }
        })?;
        let mutation_handle = match control {
            CheckedControlHandle::CheckBox(handle) => handle,
            CheckedControlHandle::RadioButton(handle) => handle,
        };
        self.turbo_vision_after_data_mutation(LiveDataMutation::SetChecked {
            handle: mutation_handle,
        });
        Ok(())
    }
    ///
    /// **Documentation:** `docs/pascal/std/tui/app/controls.md`
    pub(super) fn turbo_vision_set_items(&mut self, line: SourceLocation) -> Result<(), VmError> {
        let items = self.pop_turbo_vision_string_array("SetItems items", line)?;
        let list_handle = self.pop_turbo_vision_list_box_handle(line)?;
        self.pop_tui_application(line)?;

        let selection = self.with_tui(|tui| {
            let Some(TurboVisionObject::ListBox(list_box)) =
                tui.turbo_vision.objects.get_mut(&list_handle)
            else {
                return Err(unknown_handle_error("ListBox", list_handle, line));
            };
            let selection = initial_list_selection(&items);
            list_box.items = items.clone();
            list_box.selection_cell.set(selection);
            Ok(selection)
        })?;
        self.turbo_vision_after_data_mutation(LiveDataMutation::SetListItems {
            handle: list_handle,
            items,
            selection,
        });
        Ok(())
    }

    pub(in crate::vm::execute::io::tui) fn pop_turbo_vision_string_array(
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

pub(in crate::vm::execute::io::tui) fn initial_list_selection(items: &[String]) -> Option<usize> {
    if items.is_empty() { None } else { Some(0) }
}

impl TurboVisionChildHandle {
    fn handle(&self) -> u32 {
        match self {
            Self::Button(handle)
            | Self::StaticText(handle)
            | Self::Memo(handle)
            | Self::TextViewer(handle)
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
            Self::Memo(_) => "Memo",
            Self::TextViewer(_) => "TextViewer",
            Self::InputLine(_) => "InputLine",
            Self::ListBox(_) => "ListBox",
            Self::CheckBox(_) => "CheckBox",
            Self::RadioButton(_) => "RadioButton",
        }
    }
}
