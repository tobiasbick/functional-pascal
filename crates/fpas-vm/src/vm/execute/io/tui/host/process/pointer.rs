//! Pointer modal filtering, widget routing, and fallback callbacks.

use crate::vm::Worker;
use crate::vm::diagnostics::VmError;
use fpas_bytecode::{SourceLocation, Value};
use fpas_std::{DamageRegion, UiMouse, ViewId};

use super::DispatchTags;

impl Worker {
    /// Routes one mouse event through modal filtering, widgets, and Pascal fallback.
    pub(super) fn dispatch_tui_mouse_event(
        &mut self,
        mouse: UiMouse,
        on_mouse: Option<Value>,
        app_rec: Value,
        modal_scope: Option<&[ViewId]>,
        line: SourceLocation,
    ) -> Result<i64, VmError> {
        if self.modal_blocks_mouse_dispatch(modal_scope, mouse) {
            return Ok(19);
        }
        if let Some(tag) = self.try_dispatch_widget_mouse(mouse, modal_scope, line)? {
            return Ok(tag);
        }
        let redraw_hint = self.mouse_redraw_hint(modal_scope, mouse);
        self.dispatch_console_event_handler(
            on_mouse,
            [app_rec, Self::console_mouse_event_record(mouse)],
            Some(redraw_hint),
            DispatchTags { hit: 5, miss: 7 },
            line,
        )
    }

    fn modal_blocks_mouse_dispatch(&self, modal_scope: Option<&[ViewId]>, mouse: UiMouse) -> bool {
        let Some(scope) = modal_scope else {
            return false;
        };

        let tui = self.shared.tui.lock().unwrap_or_else(|e| e.into_inner());
        !scope.iter().any(|view_id| {
            tui.views
                .rect(*view_id)
                .is_some_and(|rect| rect.contains_console_mouse(mouse.x, mouse.y))
        })
    }

    fn mouse_redraw_hint(&self, modal_scope: Option<&[ViewId]>, mouse: UiMouse) -> DamageRegion {
        let tui = self.shared.tui.lock().unwrap_or_else(|e| e.into_inner());
        tui.views
            .topmost_view_at(
                mouse.x.saturating_sub(1),
                mouse.y.saturating_sub(1),
                modal_scope,
            )
            .and_then(|view_id| tui.views.rect(view_id))
            .map(DamageRegion::Rect)
            .unwrap_or(DamageRegion::FullFrame)
    }
}
