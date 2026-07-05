//! Live turbo-vision `Application` session owned for one FPAS `Application.Open` … `Close` cycle.
//!
//! The session lives on the main [`Worker`](crate::vm::Worker) only (`!Send`). Interactive
//! modals and `Application.Run` share one upstream app instead of calling
//! `Application::new()` per `ExecDialog` / `RunFileDialog`.
//!
//! **Documentation:** `docs/refactor/tui-bridge/done/02-single-tv-session.md`

use crate::vm::Worker;
use crate::vm::diagnostics::{VmError, runtime_error};
use fpas_bytecode::SourceLocation;
use fpas_diagnostics::codes::RUNTIME_CONSOLE_STATE_ERROR;
use turbo_vision::app::Application as TurboVisionApplication;
use turbo_vision::core::event::{Event, EventType};

impl Worker {
    /// Returns `true` when a live turbo-vision application session is active.
    pub(in crate::vm::execute::io::tui) fn turbo_vision_live_app_active(&self) -> bool {
        self.live_turbo_vision_app.is_some()
    }

    /// Create or reuse the live turbo-vision application for this FPAS session.
    pub(in crate::vm::execute::io::tui) fn turbo_vision_ensure_live_app(
        &mut self,
        line: SourceLocation,
    ) -> Result<(), VmError> {
        if self.with_tui(|tui| tui.session.is_headless()) {
            return Ok(());
        }
        if self.turbo_vision_live_app_active() {
            return Ok(());
        }

        let mut app = TurboVisionApplication::new().map_err(|error| {
            runtime_error(
                RUNTIME_CONSOLE_STATE_ERROR,
                format!("Turbo Vision terminal initialization failed: {error}"),
                "Run the program from an interactive terminal or use `Application.OpenForTest` in automated tests.",
                line,
            )
        })?;
        self.turbo_vision_sync_chrome_from_fpas(&mut app);
        self.turbo_vision_populate_desktop(&mut app);
        self.live_turbo_vision_app = Some(app);
        Ok(())
    }

    /// Run `action` with the live turbo-vision application when not headless.
    pub(in crate::vm::execute::io::tui) fn turbo_vision_with_live_app<R>(
        &mut self,
        line: SourceLocation,
        action: impl FnOnce(&mut TurboVisionApplication) -> Result<R, VmError>,
    ) -> Result<R, VmError> {
        self.turbo_vision_ensure_live_app(line)?;
        if self.with_tui(|tui| tui.session.is_headless()) {
            return Err(runtime_error(
                RUNTIME_CONSOLE_STATE_ERROR,
                "Turbo Vision live session is unavailable in headless mode",
                "Use `Application.OpenForTest` test stubs such as `Application.TestSetDialogResult`.",
                line,
            ));
        }

        let Some(app) = self.live_turbo_vision_app.as_mut() else {
            return Err(runtime_error(
                RUNTIME_CONSOLE_STATE_ERROR,
                "Turbo Vision live session is not initialized",
                "Call `Application.Open()` before using interactive Turbo Vision APIs.",
                line,
            ));
        };
        action(app)
    }

    /// Refresh chrome and desktop on an existing live session (e.g. at `Run` start).
    pub(in crate::vm::execute::io::tui) fn turbo_vision_refresh_live_desktop(
        &mut self,
        line: SourceLocation,
    ) -> Result<(), VmError> {
        self.turbo_vision_ensure_live_app(line)?;
        self.turbo_vision_rebuild_live_app_out_of_lock()
    }

    /// Rebuild the live desktop while the app is temporarily out of the worker field.
    fn turbo_vision_rebuild_live_app_out_of_lock(&mut self) -> Result<(), VmError> {
        let mut app = self.live_turbo_vision_app.take();
        if let Some(ref mut app) = app {
            self.turbo_vision_rebuild_desktop(app);
        }
        self.live_turbo_vision_app = app;
        Ok(())
    }

    /// Shut down the live turbo-vision application and release the terminal.
    pub(in crate::vm::execute::io::tui) fn turbo_vision_shutdown_live_app(&mut self) {
        if let Some(mut app) = self.live_turbo_vision_app.take() {
            let _ = app.terminal.shutdown();
        }
    }

    /// Whether the live turbo-vision application is still running its event loop.
    pub(in crate::vm::execute::io::tui) fn turbo_vision_live_app_running(&self) -> bool {
        self.live_turbo_vision_app
            .as_ref()
            .is_some_and(|app| app.running)
    }

    /// Mark the live turbo-vision application running before the interactive loop.
    pub(in crate::vm::execute::io::tui) fn turbo_vision_set_live_app_running(
        &mut self,
        running: bool,
    ) {
        if let Some(app) = self.live_turbo_vision_app.as_mut() {
            app.running = running;
        }
    }

    /// Poll the next terminal event from the live session.
    pub(in crate::vm::execute::io::tui) fn turbo_vision_live_next_event(
        &mut self,
    ) -> Option<Event> {
        self.live_turbo_vision_app
            .as_mut()
            .and_then(|app| app.get_event())
    }

    /// Dispatch one event through the live turbo-vision application.
    pub(in crate::vm::execute::io::tui) fn turbo_vision_live_handle_event(
        &mut self,
        event: &mut Event,
    ) {
        if let Some(app) = self.live_turbo_vision_app.as_mut() {
            app.handle_event(event);
        }
    }

    /// Desktop housekeeping after one interactive loop turn.
    pub(in crate::vm::execute::io::tui) fn turbo_vision_live_after_step(&mut self) {
        if let Some(app) = self.live_turbo_vision_app.as_mut() {
            let _ = app.desktop.remove_closed_windows();
            let _ = app.desktop.handle_moved_windows(&mut app.terminal);
        }
    }

    /// Mirror FPAS widget mutations after one interactive loop turn.
    pub(in crate::vm::execute::io::tui) fn turbo_vision_live_reconcile(
        &mut self,
        _line: SourceLocation,
    ) -> Result<(), VmError> {
        let dirty = self.with_tui(|tui| {
            let dirty = tui.turbo_vision.pending_reconcile.read();
            tui.turbo_vision.pending_reconcile.set(false);
            dirty
        });
        if dirty {
            self.turbo_vision_rebuild_live_app_out_of_lock()?;
        }
        Ok(())
    }

    /// Drive the live turbo-vision loop with per-step borrows so FPAS handlers can
    /// re-enter via `ExecDialog` / `RunFileDialog` on the same session.
    pub(in crate::vm::execute::io::tui) fn turbo_vision_drive_live_interactive_loop(
        &mut self,
        line: SourceLocation,
    ) -> Result<(), VmError> {
        self.turbo_vision_ensure_live_app(line)?;
        self.turbo_vision_set_live_app_running(true);

        loop {
            if !self.turbo_vision_live_app_running()
                || self.with_tui(|tui| tui.quit_requested || tui.turbo_vision.quit_requested)
            {
                return Ok(());
            }

            let Some(mut event) = self.turbo_vision_live_next_event() else {
                continue;
            };

            self.turbo_vision_live_handle_event(&mut event);

            if event.what == EventType::Command {
                self.dispatch_turbo_vision_command_event(&Event::command(event.command), line)?;
            } else if event.what != EventType::Nothing {
                self.dispatch_turbo_vision_unhandled_input(&mut event, line)?;
            }

            self.turbo_vision_live_reconcile(line)?;
            self.turbo_vision_live_after_step();
        }
    }
}
