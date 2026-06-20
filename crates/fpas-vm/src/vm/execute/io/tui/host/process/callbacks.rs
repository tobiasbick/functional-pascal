//! Invoke Pascal event callbacks with scoped redraw hints.

use crate::vm::Worker;
use crate::vm::diagnostics::VmError;
use fpas_bytecode::{SourceLocation, Value};
use fpas_std::DamageRegion;

use super::DispatchTags;

impl Worker {
    /// Dispatches a `Std.Console.Event`-bearing handler.
    pub(super) fn dispatch_console_event_handler(
        &mut self,
        handler: Option<Value>,
        args: [Value; 2],
        redraw_hint: Option<DamageRegion>,
        tags: DispatchTags,
        line: SourceLocation,
    ) -> Result<i64, VmError> {
        if let Some(handler) = handler {
            let _ = self.call_function_sync_allowing_shutdown_with_redraw_hint(
                &handler,
                &args,
                redraw_hint,
                line,
            )?;
            Ok(tags.hit)
        } else {
            Ok(tags.miss)
        }
    }

    fn call_function_sync_allowing_shutdown_with_redraw_hint(
        &mut self,
        handler: &Value,
        args: &[Value],
        redraw_hint: Option<DamageRegion>,
        line: SourceLocation,
    ) -> Result<Value, VmError> {
        {
            let mut tui = self.shared.tui.lock().unwrap_or_else(|e| e.into_inner());
            if let Some(damage) = redraw_hint {
                tui.session.set_host_redraw_hint(damage);
            } else {
                tui.session.clear_host_redraw_hint();
            }
        }

        let result = self.call_function_sync_allowing_shutdown(handler, args, line);

        let mut tui = self.shared.tui.lock().unwrap_or_else(|e| e.into_inner());
        tui.session.clear_host_redraw_hint();

        result
    }

    /// Returns a redraw hint scoped to the focused view when one exists.
    pub(super) fn focused_view_redraw_hint(&self) -> DamageRegion {
        let tui = self.shared.tui.lock().unwrap_or_else(|e| e.into_inner());
        tui.views
            .focused_id()
            .and_then(|view_id| tui.views.rect(view_id))
            .map(DamageRegion::Rect)
            .unwrap_or(DamageRegion::FullFrame)
    }
}
