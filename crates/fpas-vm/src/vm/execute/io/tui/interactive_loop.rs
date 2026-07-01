//! Turbo Vision interactive run loop and event-source seam.
//!
//! **Documentation:** `docs/future/turbo-vision-4-rust/07-post-migration-improvements.md` (Phase F/G)

use crate::vm::Worker;
use crate::vm::diagnostics::VmError;
use fpas_bytecode::SourceLocation;
use turbo_vision::app::Application as TurboVisionApplication;
use turbo_vision::core::event::{Event, EventType};

#[cfg(test)]
use std::collections::VecDeque;

/// One interactive `Application.Run` step: event intake, Turbo Vision dispatch, FPAS reconcile.
pub(crate) trait TurboVisionInteractiveSession {
    /// Mark the session running before the loop starts.
    fn set_running(&mut self, running: bool);

    /// Whether the Turbo Vision application session should keep pumping events.
    fn is_running(&self) -> bool;

    /// Return the next event to feed into Turbo Vision, if any.
    fn next_event(&mut self) -> Option<Event>;

    /// Run Turbo Vision's `handle_event` equivalent for one event.
    fn handle_event(&mut self, event: &mut Event);

    /// Desktop housekeeping after each loop turn.
    fn after_step(&mut self);

    /// Mirror FPAS-side widget mutations after a command was dispatched.
    fn reconcile(&mut self, worker: &mut Worker, line: SourceLocation) -> Result<(), VmError>;
}

/// Production session backed by a live Turbo Vision `Application`.
pub(in crate::vm::execute::io::tui) struct ApplicationInteractiveSession<'a> {
    app: &'a mut TurboVisionApplication,
}

impl<'a> ApplicationInteractiveSession<'a> {
    pub(in crate::vm::execute::io::tui) fn new(app: &'a mut TurboVisionApplication) -> Self {
        Self { app }
    }
}

impl TurboVisionInteractiveSession for ApplicationInteractiveSession<'_> {
    fn set_running(&mut self, running: bool) {
        self.app.running = running;
    }

    fn is_running(&self) -> bool {
        self.app.running
    }

    fn next_event(&mut self) -> Option<Event> {
        self.app.get_event()
    }

    fn handle_event(&mut self, event: &mut Event) {
        self.app.handle_event(event);
    }

    fn after_step(&mut self) {
        let _ = self.app.desktop.remove_closed_windows();
        let _ = self
            .app
            .desktop
            .handle_moved_windows(&mut self.app.terminal);
    }

    fn reconcile(&mut self, worker: &mut Worker, line: SourceLocation) -> Result<(), VmError> {
        worker.turbo_vision_reconcile_after_step(Some(self.app), line)
    }
}

/// Scripted events for tests without a real terminal.
#[cfg(test)]
pub(crate) struct ScriptedInteractiveSession {
    running: bool,
    events: VecDeque<Event>,
}

#[cfg(test)]
impl ScriptedInteractiveSession {
    pub(crate) fn new(events: Vec<Event>) -> Self {
        Self {
            running: true,
            events: events.into(),
        }
    }
}

#[cfg(test)]
impl TurboVisionInteractiveSession for ScriptedInteractiveSession {
    fn set_running(&mut self, running: bool) {
        self.running = running;
    }

    fn is_running(&self) -> bool {
        self.running
    }

    fn next_event(&mut self) -> Option<Event> {
        self.events.pop_front()
    }

    fn handle_event(&mut self, _event: &mut Event) {
        // Empty Turbo Vision tree: application commands remain for FPAS dispatch.
    }

    fn after_step(&mut self) {}

    fn reconcile(&mut self, worker: &mut Worker, line: SourceLocation) -> Result<(), VmError> {
        worker.turbo_vision_reconcile_after_step(None, line)
    }
}

impl Worker {
    /// Drive the live Turbo Vision loop through a pluggable interactive session.
    pub(in crate::vm::execute::io::tui) fn turbo_vision_drive_interactive_loop(
        &mut self,
        session: &mut dyn TurboVisionInteractiveSession,
        line: SourceLocation,
    ) -> Result<(), VmError> {
        session.set_running(true);
        loop {
            if !session.is_running()
                || self.with_tui(|tui| tui.quit_requested || tui.turbo_vision.quit_requested)
            {
                return Ok(());
            }

            if let Some(mut event) = session.next_event() {
                session.handle_event(&mut event);
                if event.what == EventType::Command {
                    self.dispatch_turbo_vision_command_event(&Event::command(event.command), line)?;
                    session.reconcile(self, line)?;
                } else if event.what != EventType::Nothing {
                    self.dispatch_turbo_vision_unhandled_input(&mut event, line)?;
                }
            }

            session.after_step();
        }
    }

    /// Test hook: run the interactive loop from a scripted event source.
    #[cfg(test)]
    pub(crate) fn turbo_vision_drive_interactive_loop_for_tests(
        &mut self,
        session: &mut dyn TurboVisionInteractiveSession,
        line: SourceLocation,
    ) -> Result<(), VmError> {
        self.turbo_vision_drive_interactive_loop(session, line)
    }

    /// Test hook: drive the interactive loop with scripted Turbo Vision events.
    #[cfg(test)]
    pub(crate) fn turbo_vision_drive_scripted_interactive_loop_for_tests(
        &mut self,
        events: Vec<Event>,
        line: SourceLocation,
    ) -> Result<(), VmError> {
        let mut session = ScriptedInteractiveSession::new(events);
        self.turbo_vision_drive_interactive_loop_for_tests(&mut session, line)
    }
}
