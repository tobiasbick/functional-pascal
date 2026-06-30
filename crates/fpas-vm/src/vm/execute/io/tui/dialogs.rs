//! Turbo Vision dialog construction bridge.
//!
//! **Documentation:** `docs/pascal/std/tui/app/vm-bridge.md`

use super::tv_geometry::state_rect;
use crate::vm::Worker;
use crate::vm::diagnostics::VmError;
use crate::vm::shared::{TurboVisionDialog, TurboVisionObject};
use fpas_bytecode::SourceLocation;
use turbo_vision::views::dialog::Dialog;

impl Worker {
    pub(super) fn turbo_vision_create_dialog(
        &mut self,
        line: SourceLocation,
    ) -> Result<(), VmError> {
        let title = self.pop_turbo_vision_string("Dialog title", line)?;
        let bounds = self.pop_turbo_vision_rect(line)?;
        self.pop_tui_application(line)?;

        let _dialog = Dialog::new_modal(bounds, &title);
        let bounds = state_rect(bounds);
        let handle = self.with_tui(|tui| {
            let handle = tui.turbo_vision.next_handle;
            tui.turbo_vision.next_handle = handle.saturating_add(1).max(1);
            tui.turbo_vision.objects.insert(
                handle,
                TurboVisionObject::Dialog(TurboVisionDialog {
                    bounds,
                    title,
                    children: Vec::new(),
                }),
            );
            handle
        });
        self.push(Self::turbo_vision_dialog_record(handle))
    }
}
