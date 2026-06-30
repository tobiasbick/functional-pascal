//! Turbo Vision button construction and parent attachment bridge.
//!
//! **Documentation:** `docs/pascal/std/tui/app/vm-bridge.md`

use super::handles::TurboVisionParentHandle;
use super::tv_geometry::unknown_handle_error;
use crate::vm::Worker;
use crate::vm::diagnostics::{VmError, runtime_error};
use crate::vm::shared::TurboVisionObject;
use fpas_bytecode::SourceLocation;
use fpas_diagnostics::codes::RUNTIME_INTRINSIC_STACK_STATE_ERROR;
use turbo_vision::views::button::Button;

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

    pub(super) fn turbo_vision_add_child(&mut self, line: SourceLocation) -> Result<(), VmError> {
        let button_handle = self.pop_turbo_vision_button_handle(line)?;
        let parent = self.pop_turbo_vision_parent_handle(line)?;
        self.pop_tui_application(line)?;

        self.with_tui(|tui| {
            let Some(TurboVisionObject::Button(button)) =
                tui.turbo_vision.objects.get(&button_handle)
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

            match parent {
                TurboVisionParentHandle::Dialog(dialog_handle) => {
                    let Some(TurboVisionObject::Dialog(dialog)) =
                        tui.turbo_vision.objects.get_mut(&dialog_handle)
                    else {
                        return Err(unknown_handle_error("Dialog", dialog_handle, line));
                    };
                    dialog.children.push(button_handle);
                }
                TurboVisionParentHandle::Window(window_handle) => {
                    let Some(TurboVisionObject::Window(window)) =
                        tui.turbo_vision.objects.get_mut(&window_handle)
                    else {
                        return Err(unknown_handle_error("Window", window_handle, line));
                    };
                    window.children.push(button_handle);
                }
            }

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
