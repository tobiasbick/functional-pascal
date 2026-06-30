//! Global `OnPaint` dispatch for the hosted `Std.Tui` loop.
//!
//! **Documentation:** `docs/pascal/std/tui/app/README.md` (from the repository root).

use crate::vm::Worker;
use crate::vm::diagnostics::VmError;
use fpas_bytecode::{SourceLocation, Value};
use fpas_std::{DamageRegion, ViewRect};

impl Worker {
    /// Consumes pending damage and runs the registered global `OnPaint` handler.
    ///
    /// Returns `0` when no damage exists, `5` after painting, and `6` when damage had no handler.
    pub(crate) fn tui_host_dispatch_redraw_inner(
        &mut self,
        line: SourceLocation,
    ) -> Result<i64, VmError> {
        let (damage, on_paint) = {
            let tui = self.shared.tui.lock().unwrap_or_else(|e| e.into_inner());
            (tui.session.peek_redraw_damage(line)?, tui.on_paint.clone())
        };

        let Some(expected_damage) = damage else {
            return Ok(0);
        };

        let Some(handler) = on_paint else {
            let mut tui = self.shared.tui.lock().unwrap_or_else(|e| e.into_inner());
            let consumed_damage = tui.session.take_redraw_damage(line)?;
            debug_assert_eq!(consumed_damage, Some(expected_damage));
            return Ok(6);
        };

        {
            let mut tui = self.shared.tui.lock().unwrap_or_else(|e| e.into_inner());
            let consumed_damage = tui.session.take_redraw_damage(line)?;
            debug_assert_eq!(consumed_damage, Some(expected_damage));
            self.with_console(|console| {
                tui.session
                    .begin_hosted_paint(console, expected_damage, line)
            })?;
        }

        let paint_result = self.dispatch_global_paint(handler, expected_damage, line);

        {
            let tui = self.shared.tui.lock().unwrap_or_else(|e| e.into_inner());
            self.with_console(|console| {
                if paint_result.is_ok() {
                    tui.session.finish_hosted_paint(console, line)
                } else {
                    tui.session.abort_hosted_paint(console);
                    Ok(())
                }
            })?;
        }
        paint_result?;
        Ok(5)
    }

    /// Runs global `OnPaint` with absolute screen coordinates and a hard clip at `damage`.
    fn dispatch_global_paint(
        &mut self,
        handler: Value,
        damage: DamageRegion,
        line: SourceLocation,
    ) -> Result<(), VmError> {
        let began = self.with_console(|console| {
            let screen = ViewRect {
                x: 0,
                y: 0,
                width: console.screen_width(),
                height: console.screen_height(),
            };
            let clip = match damage {
                DamageRegion::FullFrame => screen,
                DamageRegion::Rect(dirty) => dirty,
            };
            Ok(console.begin_tui_view_paint(screen, clip))
        })?;
        if !began {
            return Ok(());
        }
        let result = self.call_function_sync_allowing_shutdown(
            &handler,
            &[Self::tui_application_record()],
            line,
        );
        self.with_console(|console| {
            console.end_tui_view_paint();
            Ok(())
        })?;
        result.map(|_| ())
    }
}
