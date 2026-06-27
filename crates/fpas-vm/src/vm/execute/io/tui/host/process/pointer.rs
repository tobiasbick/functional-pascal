//! Pointer modal filtering, widget routing, and fallback callbacks.

use crate::vm::Worker;
use crate::vm::diagnostics::VmError;
use fpas_bytecode::{SourceLocation, Value};
use fpas_std::{
    BlockedInput, DamageRegion, EventOutcome, ProcessOutcome, RoutedEvent, UiMouse, ViewId,
    mouse_action_index,
};

use super::DispatchOutcomes;

impl Worker {
    /// Routes one mouse event through modal filtering, widgets, and Pascal fallback.
    pub(super) fn dispatch_tui_mouse_event(
        &mut self,
        mouse: UiMouse,
        on_mouse: Option<Value>,
        app_rec: Value,
        modal_scope: Option<&[ViewId]>,
        line: SourceLocation,
    ) -> Result<ProcessOutcome, VmError> {
        let route = {
            let tui = self.shared.tui.lock().unwrap_or_else(|e| e.into_inner());
            tui.views
                .route_event(RoutedEvent::Mouse(mouse), modal_scope)
        };

        self.sync_menu_bar_hover_outside_pointer(mouse, modal_scope, line)?;
        if let Some(tag) = self.try_dispatch_widget_mouse(mouse, modal_scope, line)? {
            return Ok(tag);
        }
        if modal_scope.is_some() && route.target.is_none() {
            return Ok(ProcessOutcome::Blocked(BlockedInput::Pointer));
        }

        if mouse.action == mouse_action_index("Down")
            && let Some(target) = route.target
        {
            let (changed, had_previous, previous, current) = {
                let mut tui = self.shared.tui.lock().unwrap_or_else(|e| e.into_inner());
                let previous = tui.views.focused_id();
                let had_previous = previous.is_some();
                let changed = tui
                    .views
                    .apply_event_outcome(&EventOutcome::RequestFocus(target));
                (changed, had_previous, previous, tui.views.focused_id())
            };
            if changed {
                self.request_focus_transition_redraw(previous, current, line)?;
                self.invoke_focus_transition(had_previous, line)?;
            }
        }
        if let Some(tag) = self.try_dispatch_control_mouse(mouse, modal_scope, line)? {
            return Ok(tag);
        }
        if let Some(tag) = self.try_dispatch_widget_wheel(mouse, modal_scope, line)? {
            return Ok(tag);
        }
        let redraw_hint = self.mouse_redraw_hint(modal_scope, mouse);
        self.dispatch_console_event_handler(
            on_mouse,
            [app_rec, Self::console_mouse_event_record(mouse)],
            Some(redraw_hint),
            DispatchOutcomes {
                hit: ProcessOutcome::Pointer { handled: true },
                miss: ProcessOutcome::Pointer { handled: false },
            },
            line,
        )
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
