//! Hosted `Std.Tui` run-loop coordination and lifecycle helpers.
//!
//! **Documentation:** `docs/pascal/std/tui-app.md` (from the repository root).

use crate::vm::Worker;
use crate::vm::diagnostics::{TYPE_MISMATCH_CODE, VmError};
use crate::vm::runtime_error;
use fpas_bytecode::{SourceLocation, Value};

const TUI_APPLICATION_TYPE: &str = "Std.Tui.Application";

impl Worker {
    /// Runs up to `max_iterations` of redraw + process-next. Stops early when both are idle
    /// or `TuiHostRequestQuit` was set.
    pub(in crate::vm::execute::io) fn tui_host_run_loop_inner(
        &mut self,
        max_iterations: usize,
        line: SourceLocation,
    ) -> Result<(), VmError> {
        const PER_EVENT_SPINS: usize = 64;
        for _ in 0..max_iterations {
            let redraw_tag = self.tui_host_dispatch_redraw_inner(line)?;
            let process_tag = self.tui_host_process_next_inner(PER_EVENT_SPINS, line)?;
            if self.take_tui_host_quit_requested() {
                break;
            }
            if redraw_tag == 0 && process_tag == 0 {
                break;
            }
        }
        Ok(())
    }

    /// Clears the quit flag and returns `true` if it was set.
    ///
    /// Clearing ensures a subsequent `TuiHostRunLoop` does not stop immediately.
    pub(in crate::vm::execute::io) fn take_tui_host_quit_requested(&self) -> bool {
        let mut tui = self.shared.tui.lock().unwrap_or_else(|e| e.into_inner());
        if tui.quit_requested {
            tui.quit_requested = false;
            true
        } else {
            false
        }
    }

    /// Converts `Application.Close(App)` into a structured host stop while `Application.Run` is active.
    ///
    /// Returns `true` when a hosted run is active (the run loop will handle the actual close).
    pub(in crate::vm::execute::io) fn request_tui_host_stop_for_active_run(&self) -> bool {
        let mut tui = self.shared.tui.lock().unwrap_or_else(|e| e.into_inner());
        if tui.run_active {
            tui.host_stop_requested = true;
            true
        } else {
            false
        }
    }

    /// Pops a `Std.Tui.Application` record from the stack, returning an error on type mismatch.
    pub(in crate::vm::execute::io) fn pop_tui_application(
        &mut self,
        line: SourceLocation,
    ) -> Result<(), VmError> {
        match self.pop(line)? {
            Value::Record { type_name, .. } if type_name == TUI_APPLICATION_TYPE => Ok(()),
            other => Err(runtime_error(
                TYPE_MISMATCH_CODE,
                format!("Expected {TUI_APPLICATION_TYPE}, got {}", other.type_name()),
                "Pass the value returned by Std.Tui.Application.Open().",
                line,
            )),
        }
    }

    /// Closes the TUI session and resets all hosted-dispatch state.
    pub(in crate::vm::execute::io) fn close_tui_application_state(
        &mut self,
        line: SourceLocation,
    ) -> Result<(), VmError> {
        let mut tui = self.shared.tui.lock().unwrap_or_else(|e| e.into_inner());
        let close_result = self.with_console_and_key_input(|console, key_input| {
            tui.session.close(console, key_input, line)
        });
        tui.host = fpas_std::UiHost::for_terminal();
        tui.on_key_pressed = None;
        tui.on_mouse = None;
        tui.on_paste = None;
        tui.on_focus_gained = None;
        tui.on_focus_lost = None;
        tui.on_activate = None;
        tui.on_deactivate = None;
        tui.on_command = None;
        tui.on_resize = None;
        tui.on_paint = None;
        tui.view_paints.clear();
        tui.view_commands.clear();
        tui.on_idle = None;
        tui.idle_interval_ms = 0;
        tui.on_exit = None;
        tui.last_exit_reason = None;
        tui.quit_requested = false;
        tui.host_stop_requested = false;
        tui.run_active = false;
        tui.views.clear();
        tui.commands.clear();
        tui.modals.clear();
        close_result?;
        Ok(())
    }
}
