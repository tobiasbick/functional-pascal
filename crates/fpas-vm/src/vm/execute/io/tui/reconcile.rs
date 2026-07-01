//! Live Turbo Vision widget-tree reconciliation during `Application.Run`.
//!
//! **Documentation:** `docs/future/turbo-vision-4-rust/07-post-migration-improvements.md` (Phase C)

use super::tv_geometry::turbo_rect;
use super::tv_run::{add_window_child, child_snapshots};
use crate::vm::Worker;
use crate::vm::diagnostics::VmError;
use crate::vm::shared::TurboVisionObject;
use fpas_bytecode::SourceLocation;
use turbo_vision::app::Application as TurboVisionApplication;
use turbo_vision::views::window::Window;

impl Worker {
    /// Record that FPAS-side Turbo Vision state changed and needs mirroring.
    pub(in crate::vm::execute::io::tui) fn mark_turbo_vision_tree_dirty(&mut self) {
        self.with_tui(|tui| tui.turbo_vision.pending_reconcile = true);
    }

    /// Reset reconcile bookkeeping at the start of `Application.Run`.
    pub(in crate::vm::execute::io::tui) fn turbo_vision_begin_run(&mut self) {
        self.with_tui(|tui| {
            tui.turbo_vision.pending_reconcile = false;
            tui.turbo_vision.live_synced_handles = tui
                .turbo_vision
                .objects
                .iter()
                .filter_map(|(handle, object)| {
                    let TurboVisionObject::Window(window) = object else {
                        return None;
                    };
                    window.on_desktop.then_some(*handle)
                })
                .collect();
        });
    }

    /// Mirror FPAS widget mutations after one Turbo Vision run step.
    pub(in crate::vm::execute::io::tui) fn turbo_vision_reconcile_after_step(
        &mut self,
        live_app: Option<&mut TurboVisionApplication>,
        line: SourceLocation,
    ) -> Result<(), VmError> {
        if self.with_tui(|tui| tui.session.is_headless()) {
            self.turbo_vision_paint_headless_desktop(line);
            return Ok(());
        }

        let dirty = self.with_tui(|tui| {
            let dirty = tui.turbo_vision.pending_reconcile;
            tui.turbo_vision.pending_reconcile = false;
            dirty
        });
        if dirty {
            if let Some(app) = live_app {
                self.turbo_vision_sync_new_windows_to_app(app)?;
            }
        }
        Ok(())
    }

    fn turbo_vision_sync_new_windows_to_app(
        &mut self,
        app: &mut TurboVisionApplication,
    ) -> Result<(), VmError> {
        let pending = self.with_tui(|tui| {
            tui.turbo_vision
                .objects
                .iter()
                .filter_map(|(handle, object)| {
                    let TurboVisionObject::Window(window) = object else {
                        return None;
                    };
                    if !window.on_desktop || tui.turbo_vision.live_synced_handles.contains(handle) {
                        return None;
                    }
                    Some((
                        *handle,
                        window.bounds,
                        window.title.clone(),
                        window.children.clone(),
                    ))
                })
                .collect::<Vec<_>>()
        });

        for (handle, bounds, title, children) in pending {
            let child_views =
                self.with_tui(|tui| child_snapshots(&tui.turbo_vision.objects, &children));
            let mut window_view = Window::new(turbo_rect(bounds), &title);
            for child in child_views {
                add_window_child(&mut window_view, child);
            }
            app.desktop.add(Box::new(window_view));
            self.with_tui(|tui| {
                tui.turbo_vision.live_synced_handles.insert(handle);
            });
        }
        Ok(())
    }
}
