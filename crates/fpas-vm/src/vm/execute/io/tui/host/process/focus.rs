//! Focus transitions and focus-related redraw invalidation.

use crate::vm::Worker;
use crate::vm::diagnostics::VmError;
use fpas_bytecode::SourceLocation;
use fpas_std::ViewId;

impl Worker {
    /// Invalidates the previous and current focused view after traversal.
    pub(super) fn request_focus_transition_redraw(
        &self,
        previous_focus: Option<ViewId>,
        current_focus: Option<ViewId>,
        line: SourceLocation,
    ) -> Result<(), VmError> {
        let mut tui = self.shared.tui.lock().unwrap_or_else(|e| e.into_inner());
        let previous_rect = previous_focus.and_then(|view_id| tui.views.rect(view_id));
        let current_rect = current_focus.and_then(|view_id| tui.views.rect(view_id));

        let mut marked_any = false;
        if let Some(rect) = previous_rect {
            tui.session.request_redraw_rect(rect, line)?;
            marked_any = true;
        }
        if let Some(rect) = current_rect {
            tui.session.request_redraw_rect(rect, line)?;
            marked_any = true;
        }
        if !marked_any {
            tui.session.request_redraw(line)?;
        }

        Ok(())
    }

    /// Fires `OnDeactivate` when requested, followed by `OnActivate`.
    pub(super) fn invoke_focus_transition(
        &mut self,
        fire_deactivate: bool,
        line: SourceLocation,
    ) -> Result<(), VmError> {
        let app_rec = Self::tui_application_record();

        if fire_deactivate {
            let handler = {
                let tui = self.shared.tui.lock().unwrap_or_else(|e| e.into_inner());
                tui.on_deactivate.clone()
            };
            if let Some(handler) = handler {
                let _ = self.call_function_sync_allowing_shutdown(
                    &handler,
                    std::slice::from_ref(&app_rec),
                    line,
                )?;
            }
        }

        let handler = {
            let tui = self.shared.tui.lock().unwrap_or_else(|e| e.into_inner());
            tui.on_activate.clone()
        };
        if let Some(handler) = handler {
            let _ = self.call_function_sync_allowing_shutdown(&handler, &[app_rec], line)?;
        }

        Ok(())
    }
}
