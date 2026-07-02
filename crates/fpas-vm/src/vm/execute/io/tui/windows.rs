//! Turbo Vision window construction and desktop attachment bridge.
//!
//! **Documentation:** `docs/pascal/std/tui/app/vm-bridge.md`

use super::handles::TurboVisionParentHandle;
use super::tv_geometry::state_rect;
use crate::vm::Worker;
use crate::vm::diagnostics::{VmError, runtime_error};
use crate::vm::shared::{TurboVisionObject, TurboVisionWindow};
use fpas_bytecode::SourceLocation;
use fpas_diagnostics::codes::RUNTIME_INTRINSIC_STACK_STATE_ERROR;
use turbo_vision::views::window::Window;

impl Worker {
    pub(super) fn turbo_vision_create_window(
        &mut self,
        line: SourceLocation,
    ) -> Result<(), VmError> {
        let title = self.pop_turbo_vision_string("Window title", line)?;
        let bounds = self.pop_turbo_vision_rect(line)?;
        self.pop_tui_application(line)?;

        let _window = Window::new(bounds, &title);
        let bounds = state_rect(bounds);
        let handle = self.with_tui(|tui| {
            let handle = tui.turbo_vision.next_handle;
            tui.turbo_vision.next_handle = handle.saturating_add(1).max(1);
            tui.turbo_vision.objects.insert(
                handle,
                TurboVisionObject::Window(TurboVisionWindow {
                    bounds,
                    title,
                    children: Vec::new(),
                    on_desktop: false,
                }),
            );
            handle
        });
        self.mark_turbo_vision_tree_dirty();
        self.push(Self::turbo_vision_window_record(handle))
    }

    pub(super) fn turbo_vision_add_window(&mut self, line: SourceLocation) -> Result<(), VmError> {
        let window_handle = self.pop_turbo_vision_window_handle(line)?;
        self.pop_tui_application(line)?;

        self.with_tui(|tui| {
            let Some(TurboVisionObject::Window(window)) =
                tui.turbo_vision.objects.get_mut(&window_handle)
            else {
                return Err(super::tv_geometry::unknown_handle_error(
                    "Window",
                    window_handle,
                    line,
                ));
            };

            if window.on_desktop {
                return Err(runtime_error(
                    RUNTIME_INTRINSIC_STACK_STATE_ERROR,
                    format!("Window handle {window_handle} is already on the desktop"),
                    "Call `Application.AddWindow` once per window.",
                    line,
                ));
            }

            window.on_desktop = true;
            Ok(())
        })?;
        self.mark_turbo_vision_tree_dirty();
        Ok(())
    }

    /// Replace the title of a window or dialog root.
    ///
    /// **Documentation:** `docs/pascal/std/tui/app/README.md`
    pub(super) fn turbo_vision_set_title(&mut self, line: SourceLocation) -> Result<(), VmError> {
        let title = self.pop_turbo_vision_string("SetTitle title", line)?;
        let root = self.pop_turbo_vision_parent_handle(line)?;
        self.pop_tui_application(line)?;

        self.with_tui(|tui| match root {
            TurboVisionParentHandle::Dialog(handle) => {
                let Some(TurboVisionObject::Dialog(dialog)) =
                    tui.turbo_vision.objects.get_mut(&handle)
                else {
                    return Err(super::tv_geometry::unknown_handle_error(
                        "Dialog", handle, line,
                    ));
                };
                dialog.title = title;
                Ok(())
            }
            TurboVisionParentHandle::Window(handle) => {
                let Some(TurboVisionObject::Window(window)) =
                    tui.turbo_vision.objects.get_mut(&handle)
                else {
                    return Err(super::tv_geometry::unknown_handle_error(
                        "Window", handle, line,
                    ));
                };
                window.title = title;
                Ok(())
            }
        })?;
        self.mark_turbo_vision_tree_dirty();
        Ok(())
    }
}
