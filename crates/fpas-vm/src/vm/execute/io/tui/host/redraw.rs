//! Depth-first retained `Std.Tui` redraw and paint dispatch.
//!
//! **Documentation:** `docs/pascal/std/tui/app/README.md` (from the repository root).

use crate::vm::Worker;
use crate::vm::diagnostics::VmError;
use fpas_bytecode::{SourceLocation, Value};
use fpas_std::{DamageRegion, ResolvedView, ViewId, ViewRect, ViewWidget};

struct DeferredOverlay {
    view_id: ViewId,
    rect: ViewRect,
}

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

        let can_run_global_paint = roots.is_empty() && !has_view_paints && !has_view_widgets;
        let has_paint =
            (on_paint.is_some() && can_run_global_paint) || has_view_paints || has_view_widgets;

        if !has_paint {
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
            if can_run_global_paint && let Some(handler) = on_paint {
                self.dispatch_global_paint(handler, expected_damage, line)?;
            }
            let mut overlays = Vec::new();
            for root in roots {
                self.paint_view_subtree(root, expected_damage, line, &mut overlays)?;
            }
            for overlay in overlays {
                self.paint_scene_overlay(&overlay, expected_damage)?;
            }
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
        overlays: &mut Vec<DeferredOverlay>,
    ) -> Result<(), VmError> {
        let snapshot = {
            let tui = self.shared.tui.lock().unwrap_or_else(|e| e.into_inner());
            let resolved = tui.views.resolved(view_id);
            let handler = tui.view_paints.get(&view_id).cloned();
            let children = tui.views.children(view_id).to_vec();
            let widget_damaged = resolved.is_some_and(|view| {
                view.state.exposed
                    && tui
                        .view_widgets
                        .get(&view_id)
                        .is_some_and(|widget| widget.intersects_damage(view.rect, damage))
            });
            (resolved, handler, children, widget_damaged)
        };
        let (resolved, handler, children, widget_damaged) = snapshot;

        let paint_view = resolved
            .filter(|view| view.state.exposed && Self::damage_intersects_view(damage, *view));

        if let Some(view) = resolved.filter(|view| view.state.exposed && widget_damaged)
            && let Some(widget) = self.take_paint_widget(view)
        {
            let result = self.paint_widget_underlay(&widget, view, damage);
            self.restore_paint_widget(view.id, widget);
            result?;
        }

        if let Some(view) = paint_view
            && let Some(handler) = handler
        {
            self.dispatch_local_paint(handler, view, line)?;
        }

        for child in children {
            self.paint_view_subtree(child, damage, line, overlays)?;
        }

        if let Some(view) = resolved.filter(|view| view.state.exposed && widget_damaged)
            && let Some(widget) = self.take_paint_widget(view)
        {
            let result = self.paint_widget_overlay(&widget, view, damage);
            if widget.has_scene_overlay() {
                overlays.push(DeferredOverlay {
                    view_id: view.id,
                    rect: view.rect,
                });
            }
            self.restore_paint_widget(view.id, widget);
            result?;
        }
        Ok(())
    }

    fn take_paint_widget(&self, view: ResolvedView) -> Option<ViewWidget> {
        self.with_tui(|tui| {
            let mut widget = tui.view_widgets.remove(&view.id)?;
            widget.sync_view_state(view.state);
            if let ViewWidget::Frame(frame) = &mut widget
                && let Some(state) = tui.views.frame_root_state(view.id).copied()
            {
                frame.content_size = state.content_size;
                frame.scroll_x = state.scroll_x;
                frame.scroll_y = state.scroll_y;
            }
            Some(widget)
        })
    }

    fn restore_paint_widget(&self, view_id: ViewId, widget: ViewWidget) {
        self.with_tui(|tui| {
            tui.view_widgets.insert(view_id, widget);
        });
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

    fn paint_scene_overlay(
        &mut self,
        overlay: &DeferredOverlay,
        damage: DamageRegion,
    ) -> Result<(), VmError> {
        let Some(widget) = self.take_scene_overlay_widget(overlay.view_id) else {
            return Ok(());
        };
        let result = self.with_console(|console| {
            let screen = ViewRect {
                x: 0,
                y: 0,
                width: console.screen_width(),
                height: console.screen_height(),
            };
            if console.begin_tui_view_paint(screen, screen) {
                widget.paint_scene_overlay(console, overlay.rect, damage);
                console.end_tui_view_paint();
            }
            Ok(())
        });
        self.restore_paint_widget(overlay.view_id, widget);
        result
    }

    fn take_scene_overlay_widget(&self, view_id: ViewId) -> Option<ViewWidget> {
        self.with_tui(|tui| tui.view_widgets.remove(&view_id))
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
