//! Live Turbo Vision widget-tree reconciliation during `Application.Run`.
//!
//! **Documentation:** `docs/pascal/std/tui/app/vm-bridge.md`

use crate::vm::Worker;
use crate::vm::diagnostics::VmError;
use fpas_bytecode::SourceLocation;
use turbo_vision::app::Application as TurboVisionApplication;

impl Worker {
    /// Record that FPAS-side Turbo Vision state changed and needs mirroring.
    pub(in crate::vm::execute::io::tui) fn mark_turbo_vision_tree_dirty(&mut self) {
        self.with_tui(|tui| tui.turbo_vision.pending_reconcile.set(true));
    }

    /// Record that the headless desktop needs redraw without structural rebuild.
    pub(in crate::vm::execute::io::tui) fn mark_turbo_vision_headless_repaint(&mut self) {
        self.with_tui(|tui| tui.turbo_vision.pending_headless_repaint.set(true));
    }

    /// Reset reconcile bookkeeping at the start of `Application.Run`.
    ///
    /// The initial desktop is built once by `build_turbo_vision_application`, so
    /// the first reconcile only needs to fire once FPAS mutates the tree.
    pub(in crate::vm::execute::io::tui) fn turbo_vision_begin_run(&mut self) {
        self.with_tui(|tui| {
            tui.turbo_vision.pending_reconcile.set(false);
            tui.turbo_vision.pending_headless_repaint.set(false);
        });
    }

    /// Mirror FPAS widget mutations after one Turbo Vision run step.
    ///
    /// When the FPAS widget tree changed, the whole desktop is rebuilt from the
    /// current FPAS state. A full rebuild (rather than an incremental add) keeps
    /// the live view correct for every structural change, including children added
    /// to roots that were already shown — an incremental "add new roots only" pass
    /// would miss those.
    pub(in crate::vm::execute::io::tui) fn turbo_vision_reconcile_after_step(
        &mut self,
        live_app: Option<&mut TurboVisionApplication>,
        line: SourceLocation,
    ) -> Result<(), VmError> {
        if self.with_tui(|tui| tui.session.is_headless()) {
            let (structural, repaint) = self.with_tui(|tui| {
                let structural = tui.turbo_vision.pending_reconcile.read();
                tui.turbo_vision.pending_reconcile.set(false);
                let repaint = tui.turbo_vision.pending_headless_repaint.read();
                tui.turbo_vision.pending_headless_repaint.set(false);
                (structural, repaint)
            });
            if structural {
                self.turbo_vision_paint_headless_desktop(line, true);
            } else if repaint {
                self.turbo_vision_paint_headless_desktop(line, false);
            }
            return Ok(());
        }

        let dirty = self.with_tui(|tui| {
            let dirty = tui.turbo_vision.pending_reconcile.read();
            tui.turbo_vision.pending_reconcile.set(false);
            dirty
        });
        if dirty && let Some(app) = live_app {
            self.turbo_vision_rebuild_desktop(app);
        }
        Ok(())
    }

    pub(in crate::vm::execute::io::tui) fn turbo_vision_rebuild_desktop(
        &self,
        app: &mut TurboVisionApplication,
    ) {
        self.turbo_vision_clear_live_view_ids();
        self.turbo_vision_sync_chrome_from_fpas(app);
        while app.desktop.child_count() > 0 {
            app.desktop.remove_child(0);
        }
        self.turbo_vision_populate_desktop(app);
    }
}
