//! Hosted `Std.Tui` redraw and paint dispatch.
//!
//! **Documentation:** `docs/pascal/std/tui-app.md` (from the repository root).

use crate::vm::Worker;
use crate::vm::diagnostics::VmError;
use fpas_bytecode::{SourceLocation, Value};
use fpas_std::{DamageRegion, ViewRect};

impl Worker {
    /// Consumes a pending redraw and invokes `OnPaint` if registered.
    ///
    /// Returns `0` = no redraw pending, `5` = `OnPaint` ran, `6` = pending but no handler (cleared).
    pub(in crate::vm::execute::io) fn tui_host_dispatch_redraw_inner(
        &mut self,
        line: SourceLocation,
    ) -> Result<i64, VmError> {
        let (damage, on_paint, has_view_paints, has_view_widgets) = {
            let tui = self.shared.tui.lock().unwrap_or_else(|e| e.into_inner());
            let damage = tui.session.peek_redraw_damage(line)?;
            (
                damage,
                tui.on_paint.clone(),
                !tui.view_paints.is_empty(),
                !tui.view_widgets.is_empty(),
            )
        };

        let Some(expected_damage) = damage else {
            return Ok(0);
        };

        let app_rec = Self::tui_application_record();

        if on_paint.is_some() || has_view_paints || has_view_widgets {
            {
                let mut tui = self.shared.tui.lock().unwrap_or_else(|e| e.into_inner());
                let consumed_damage = tui.session.take_redraw_damage(line)?;
                debug_assert_eq!(consumed_damage, Some(expected_damage));
                self.with_console(|console| {
                    tui.session
                        .begin_hosted_paint(console, expected_damage, line)
                })?;
            }
            let paint_result = (|| -> Result<(), VmError> {
                if let Some(handler) = on_paint {
                    let _ =
                        self.call_function_sync_allowing_shutdown(&handler, &[app_rec], line)?;
                }
                let _ = self.dispatch_view_widget_paints(expected_damage, line)?;
                let _ = self.dispatch_view_paint_handlers(expected_damage, line)?;
                Ok(())
            })();
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
        } else {
            let mut tui = self.shared.tui.lock().unwrap_or_else(|e| e.into_inner());
            let consumed_damage = tui.session.take_redraw_damage(line)?;
            debug_assert_eq!(consumed_damage, Some(expected_damage));
            Ok(6)
        }
    }

    fn dispatch_view_widget_paints(
        &mut self,
        damage: DamageRegion,
        _line: SourceLocation,
    ) -> Result<bool, VmError> {
        let scheduled = {
            let tui = self.shared.tui.lock().unwrap_or_else(|e| e.into_inner());
            tui.views
                .paint_order()
                .into_iter()
                .filter_map(|view_id| {
                    let widget = tui.view_widgets.get(&view_id)?.clone();
                    let rect = tui.views.rect(view_id)?;
                    Self::damage_intersects_rect(damage, rect).then_some((widget, rect))
                })
                .collect::<Vec<_>>()
        };

        if scheduled.is_empty() {
            return Ok(false);
        }

        self.with_console(|console| {
            for (widget, rect) in scheduled {
                widget.paint(console, rect, damage);
            }
            Ok(())
        })?;

        Ok(true)
    }

    fn dispatch_view_paint_handlers(
        &mut self,
        damage: DamageRegion,
        line: SourceLocation,
    ) -> Result<bool, VmError> {
        let scheduled = {
            let tui = self.shared.tui.lock().unwrap_or_else(|e| e.into_inner());
            tui.views
                .paint_order()
                .into_iter()
                .filter_map(|view_id| {
                    if tui.view_widgets.contains_key(&view_id) {
                        return None;
                    }
                    let handler = tui.view_paints.get(&view_id)?.clone();
                    let rect = tui.views.rect(view_id)?;
                    Self::damage_intersects_rect(damage, rect).then_some((view_id, rect, handler))
                })
                .collect::<Vec<_>>()
        };

        if scheduled.is_empty() {
            return Ok(false);
        }

        let app_rec = Self::tui_application_record();
        for (view_id, rect, handler) in scheduled {
            let _ = self.call_function_sync_allowing_shutdown(
                &handler,
                &[
                    app_rec.clone(),
                    Value::Integer(i64::from(view_id.raw())),
                    Self::tui_rect_record(rect),
                ],
                line,
            )?;
        }

        Ok(true)
    }

    fn damage_intersects_rect(damage: DamageRegion, rect: ViewRect) -> bool {
        match damage {
            DamageRegion::FullFrame => true,
            DamageRegion::Rect(dirty) => Self::rects_intersect(dirty, rect),
        }
    }

    fn rects_intersect(left: ViewRect, right: ViewRect) -> bool {
        let left_right = left.x.saturating_add(left.width);
        let left_bottom = left.y.saturating_add(left.height);
        let right_right = right.x.saturating_add(right.width);
        let right_bottom = right.y.saturating_add(right.height);

        left.x < right_right
            && right.x < left_right
            && left.y < right_bottom
            && right.y < left_bottom
    }
}
