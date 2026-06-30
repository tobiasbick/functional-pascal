//! Turbo Vision button construction and parent attachment bridge.
//!
//! **Documentation:** `docs/pascal/std/tui/app/vm-bridge.md`

use super::tv_geometry::{turbo_rect, unknown_handle_error};
use crate::vm::Worker;
use crate::vm::diagnostics::{VmError, runtime_error};
use crate::vm::shared::{TurboVisionButton, TurboVisionObject};
use fpas_bytecode::SourceLocation;
use fpas_diagnostics::codes::RUNTIME_INTRINSIC_STACK_STATE_ERROR;
use turbo_vision::views::{button::Button, dialog::Dialog};

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
        self.push(Self::turbo_vision_button_record(handle))
    }

    pub(super) fn turbo_vision_add_child(&mut self, line: SourceLocation) -> Result<(), VmError> {
        let button_handle = self.pop_turbo_vision_button_handle(line)?;
        let dialog_handle = self.pop_turbo_vision_dialog_handle(line)?;
        self.pop_tui_application(line)?;

        self.with_tui(|tui| {
            if !matches!(
                tui.turbo_vision.objects.get(&dialog_handle),
                Some(TurboVisionObject::Dialog(_))
            ) {
                return Err(unknown_handle_error("Dialog", dialog_handle, line));
            }

            let Some(TurboVisionObject::Button(button)) =
                tui.turbo_vision.objects.get_mut(&button_handle)
            else {
                return Err(unknown_handle_error("Button", button_handle, line));
            };

            if button.attached {
                return Err(runtime_error(
                    RUNTIME_INTRINSIC_STACK_STATE_ERROR,
                    format!("Button handle {button_handle} is already attached"),
                    "Only add a Turbo Vision button to one parent.",
                    line,
                ));
            }

            let Some(TurboVisionObject::Dialog(dialog)) =
                tui.turbo_vision.objects.get_mut(&dialog_handle)
            else {
                return Err(unknown_handle_error("Dialog", dialog_handle, line));
            };

            let mut dialog_view = Dialog::new_modal(turbo_rect(dialog.bounds), &dialog.title);
            let button_snapshot = match tui.turbo_vision.objects.get(&button_handle) {
                Some(TurboVisionObject::Button(button)) => {
                    (button.bounds, button.text.clone(), button.command_id)
                }
                _ => return Err(unknown_handle_error("Button", button_handle, line)),
            };
            dialog_view.add(Box::new(Button::new(
                turbo_rect(button_snapshot.0),
                &button_snapshot.1,
                button_snapshot.2,
                false,
            )));
            let Some(TurboVisionObject::Dialog(dialog)) =
                tui.turbo_vision.objects.get_mut(&dialog_handle)
            else {
                return Err(unknown_handle_error("Dialog", dialog_handle, line));
            };
            dialog.children.push(button_handle);
            if let Some(TurboVisionObject::Button(button)) =
                tui.turbo_vision.objects.get_mut(&button_handle)
            {
                button.attached = true;
            }
            Ok(())
        })?;
        Ok(())
    }
}
