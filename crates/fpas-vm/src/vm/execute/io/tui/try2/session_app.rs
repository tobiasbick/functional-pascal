//! Try-2 live turbo-vision `Application` session owned for one FPAS `Application.Open` … `Close` cycle.
//!
//! The session lives on the main [`Worker`](crate::vm::Worker) only (`!Send`). Try-2 modals and
//! `Application.Run` share one upstream app instead of calling `Application::new()` per modal.
//!
//! **Documentation:** `docs/pascal/std/tui/session.md`

use crate::vm::Worker;
use turbo_vision::core::event::Event;

impl Worker {
    /// Returns `true` when a live turbo-vision application session is active.
    pub(in crate::vm::execute::io::tui) fn turbo_vision_live_app_active(&self) -> bool {
        self.live_turbo_vision_app.is_some()
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
}
