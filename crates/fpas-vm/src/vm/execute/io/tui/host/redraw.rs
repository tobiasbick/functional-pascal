//! Depth-first retained `Std.Tui` redraw and paint dispatch.
//!
//! **Documentation:** `docs/pascal/std/tui/app/README.md` (from the repository root).

use crate::vm::Worker;
use crate::vm::diagnostics::VmError;
use fpas_bytecode::{SourceLocation, Value};
use fpas_std::{DamageRegion, ResolvedView, ViewId, ViewRect, ViewWidget};

impl Worker {
    /// Consumes pending damage and paints the retained view tree.
    ///
    /// Returns `0` when no damage exists, `5` after painting, and `6` when damage had no handler.
    pub(crate) fn tui_host_dispatch_redraw_inner(
        &mut self,
        line: SourceLocation,
    ) -> Result<i64, VmError> {
        let (damage, on_paint, has_view_paints, has_view_widgets, roots) = {
            let tui = self.shared.tui.lock().unwrap_or_else(|e| e.into_inner());
            (
                tui.session.peek_redraw_damage(line)?,
                tui.on_paint.clone(),
                !tui.view_paints.is_empty(),
                !tui.view_widgets.is_empty(),
                tui.views.roots().to_vec(),
            )
        };

        let Some(expected_damage) = damage else {
            return Ok(0);
        };

        if on_paint.is_none() && !has_view_paints && !has_view_widgets {
            let mut tui = self.shared.tui.lock().unwrap_or_else(|e| e.into_inner());
            let consumed_damage = tui.session.take_redraw_damage(line)?;
            debug_assert_eq!(consumed_damage, Some(expected_damage));
            return Ok(6);
        }

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
                self.dispatch_global_paint(handler, expected_damage, line)?;
            }
            for root in roots {
                self.paint_view_subtree(root, expected_damage, line)?;
            }
            self.paint_menu_overlay_layer(expected_damage)?;
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

    fn paint_view_subtree(
        &mut self,
        view_id: ViewId,
        damage: DamageRegion,
        line: SourceLocation,
    ) -> Result<(), VmError> {
        let snapshot = {
            let tui = self.shared.tui.lock().unwrap_or_else(|e| e.into_inner());
            let resolved = tui.views.resolved(view_id);
            let widget = tui.view_widgets.get(&view_id).cloned();
            let handler = tui.view_paints.get(&view_id).cloned();
            let children = tui.views.children(view_id).to_vec();
            (resolved, widget, handler, children)
        };
        let (resolved, mut widget, handler, children) = snapshot;

        let paint_view = resolved
            .filter(|view| view.state.exposed && Self::damage_intersects_view(damage, *view));
        if let Some(view) = paint_view {
            if let Some(widget) = widget.as_mut() {
                widget.sync_view_state(view.state);
                if let ViewWidget::Frame(frame) = widget {
                    if let Some(state) = {
                        let tui = self.shared.tui.lock().unwrap_or_else(|e| e.into_inner());
                        tui.views.frame_root_state(view.id).copied()
                    } {
                        frame.content_size = state.content_size;
                        frame.scroll_x = state.scroll_x;
                        frame.scroll_y = state.scroll_y;
                    }
                }
            }
            if let Some(widget) = widget.as_ref() {
                self.paint_widget_underlay(widget, view, damage)?;
            }
            if let Some(handler) = handler {
                self.dispatch_local_paint(handler, view, line)?;
            }
        }

        for child in children {
            self.paint_view_subtree(child, damage, line)?;
        }
        if let (Some(view), Some(widget)) = (paint_view, widget.as_ref()) {
            self.paint_widget_overlay(widget, view, damage)?;
        }
        Ok(())
    }

    fn paint_widget_underlay(
        &mut self,
        widget: &ViewWidget,
        view: ResolvedView,
        damage: DamageRegion,
    ) -> Result<(), VmError> {
        let Some(clip) = view.clip else {
            return Ok(());
        };
        self.with_console(|console| {
            if console.begin_tui_view_paint(view.rect, clip) {
                widget.paint_underlay(console, view.rect, damage);
                console.end_tui_view_paint();
            }
            Ok(())
        })
    }

    fn paint_widget_overlay(
        &mut self,
        widget: &ViewWidget,
        view: ResolvedView,
        damage: DamageRegion,
    ) -> Result<(), VmError> {
        let Some(clip) = view.clip else {
            return Ok(());
        };
        self.with_console(|console| {
            if console.begin_tui_view_paint(view.rect, clip) {
                widget.paint_overlay(console, view.rect, damage);
                console.end_tui_view_paint();
            }
            Ok(())
        })
    }

    fn dispatch_local_paint(
        &mut self,
        handler: Value,
        view: ResolvedView,
        line: SourceLocation,
    ) -> Result<(), VmError> {
        let Some(clip) = view.clip else {
            return Ok(());
        };
        let began =
            self.with_console(|console| Ok(console.begin_tui_view_paint(view.rect, clip)))?;
        if !began {
            return Ok(());
        }

        let local_bounds = ViewRect {
            x: 0,
            y: 0,
            width: view.rect.width,
            height: view.rect.height,
        };
        let result = self.call_function_sync_allowing_shutdown(
            &handler,
            &[
                Self::tui_application_record(),
                Self::tui_view_id_record(view.id),
                Self::tui_rect_record(local_bounds),
            ],
            line,
        );
        self.with_console(|console| {
            console.end_tui_view_paint();
            Ok(())
        })?;
        result.map(|_| ())
    }

    fn paint_menu_overlay_layer(&mut self, damage: DamageRegion) -> Result<(), VmError> {
        let scheduled = {
            let tui = self.shared.tui.lock().unwrap_or_else(|e| e.into_inner());
            tui.views
                .resolved_paint_order()
                .into_iter()
                .filter_map(|view| {
                    let widget = tui.view_widgets.get(&view.id)?;
                    matches!(widget, ViewWidget::MenuBar(_)).then_some((widget.clone(), view.rect))
                })
                .collect::<Vec<_>>()
        };

        self.with_console(|console| {
            let screen = ViewRect {
                x: 0,
                y: 0,
                width: console.screen_width(),
                height: console.screen_height(),
            };
            for (widget, rect) in &scheduled {
                if console.begin_tui_view_paint(screen, screen) {
                    widget.paint_menu_overlays(console, *rect, damage);
                    console.end_tui_view_paint();
                }
            }
            Ok(())
        })
    }

    fn damage_intersects_view(damage: DamageRegion, view: ResolvedView) -> bool {
        let Some(clip) = view.clip else {
            return false;
        };
        match damage {
            DamageRegion::FullFrame => true,
            DamageRegion::Rect(dirty) => dirty.intersects(clip),
        }
    }
}
