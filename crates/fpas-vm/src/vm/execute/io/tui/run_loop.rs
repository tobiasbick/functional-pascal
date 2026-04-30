//! `Std.Tui` run loop, event dispatch, and session lifecycle.
//!
//! **Documentation:** `docs/pascal/std/tui-app.md` (from the repository root).

use crate::vm::diagnostics::{TYPE_MISMATCH_CODE, VmError};
use crate::vm::{Worker, runtime_error};
use fpas_bytecode::{SourceLocation, Value};
use fpas_std::HostEvent;

const TUI_APPLICATION_TYPE: &str = "Std.Tui.Application";

impl Worker {
    /// Processes at most one pending `HostEvent`, dispatching to the registered handler.
    ///
    /// Returns a status tag: `0` = none, `1` = key dispatched, `2` = resize dispatched,
    /// `3` = key (no handler), `4` = resize (no handler), `5`/`7`/`8`/`9`/`10`/`11`/`12`/`13`
    /// for mouse/paste/focus events (dispatched or not).
    pub(in crate::vm::execute::io) fn tui_host_process_next_inner(
        &mut self,
        max_spins: usize,
        line: SourceLocation,
    ) -> Result<i64, VmError> {
        let mut ready: Option<HostEvent> = None;
        for _ in 0..max_spins.max(1) {
            let mut tui = self.shared.tui.lock().unwrap_or_else(|e| e.into_inner());
            if let Some(ev) = tui.host.pop_ready_event() {
                ready = Some(ev);
                break;
            }
            let polled = self.with_console_and_key_input(|console, key_input| {
                tui.session.poll_event_all(console, key_input, line)
            })?;
            match polled {
                None => break,
                Some(tui_ev) => {
                    tui.host.ingest_tui_event(tui_ev);
                    if let Some(ev) = tui.host.pop_ready_event() {
                        ready = Some(ev);
                        break;
                    }
                }
            }
        }

        let Some(ev) = ready else {
            return Ok(0);
        };

        let (on_key, on_mouse, on_paste, on_focus_gained, on_focus_lost, on_resize) = {
            let tui = self.shared.tui.lock().unwrap_or_else(|e| e.into_inner());
            (
                tui.on_key_pressed.clone(),
                tui.on_mouse.clone(),
                tui.on_paste.clone(),
                tui.on_focus_gained.clone(),
                tui.on_focus_lost.clone(),
                tui.on_resize.clone(),
            )
        };

        let app_rec = Self::tui_application_record();

        match ev {
            HostEvent::Key(k) => {
                if let Some(handler) = on_key {
                    let _ = self.call_function_sync(
                        &handler,
                        &[app_rec, Self::key_event_record(k)],
                        line,
                    )?;
                    Ok(1)
                } else {
                    Ok(3)
                }
            }
            HostEvent::Mouse(mouse_ev) => {
                if let Some(handler) = on_mouse {
                    let _ = self.call_function_sync(
                        &handler,
                        &[app_rec, Self::console_event_record(mouse_ev)],
                        line,
                    )?;
                    Ok(5)
                } else {
                    Ok(7)
                }
            }
            HostEvent::Paste(paste_ev) => {
                if let Some(handler) = on_paste {
                    let _ = self.call_function_sync(
                        &handler,
                        &[app_rec, Self::console_event_record(paste_ev)],
                        line,
                    )?;
                    Ok(8)
                } else {
                    Ok(9)
                }
            }
            HostEvent::FocusGained(focus_ev) => {
                if let Some(handler) = on_focus_gained {
                    let _ = self.call_function_sync(
                        &handler,
                        &[app_rec, Self::console_event_record(focus_ev)],
                        line,
                    )?;
                    Ok(10)
                } else {
                    Ok(11)
                }
            }
            HostEvent::FocusLost(focus_ev) => {
                if let Some(handler) = on_focus_lost {
                    let _ = self.call_function_sync(
                        &handler,
                        &[app_rec, Self::console_event_record(focus_ev)],
                        line,
                    )?;
                    Ok(12)
                } else {
                    Ok(13)
                }
            }
            HostEvent::Resize { width, height } => {
                if let Some(handler) = on_resize {
                    let _ = self.call_function_sync(
                        &handler,
                        &[app_rec, Self::tui_size_record(width, height)],
                        line,
                    )?;
                    Ok(2)
                } else {
                    Ok(4)
                }
            }
        }
    }

    /// Consumes a pending redraw and invokes `OnPaint` if registered.
    ///
    /// Returns `0` = no redraw pending, `5` = `OnPaint` ran, `6` = pending but no handler (cleared).
    pub(in crate::vm::execute::io) fn tui_host_dispatch_redraw_inner(
        &mut self,
        line: SourceLocation,
    ) -> Result<i64, VmError> {
        let (pending, on_paint) = {
            let tui = self.shared.tui.lock().unwrap_or_else(|e| e.into_inner());
            let pending = tui.session.is_redraw_pending(line)?;
            (pending, tui.on_paint.clone())
        };

        if !pending {
            return Ok(0);
        }

        let app_rec = Self::tui_application_record();

        if let Some(handler) = on_paint {
            {
                let mut tui = self.shared.tui.lock().unwrap_or_else(|e| e.into_inner());
                let _ = tui.session.take_redraw_pending(line)?;
            }
            let _ = self.call_function_sync(&handler, &[app_rec], line)?;
            Ok(5)
        } else {
            let mut tui = self.shared.tui.lock().unwrap_or_else(|e| e.into_inner());
            let _ = tui.session.take_redraw_pending(line)?;
            Ok(6)
        }
    }

    /// Runs up to `max_iterations` of redraw + process-next. Stops early when both are idle
    /// or `TuiHostRequestQuit` was set.
    pub(in crate::vm::execute::io) fn tui_host_run_loop_inner(
        &mut self,
        max_iterations: usize,
        line: SourceLocation,
    ) -> Result<(), VmError> {
        const PER_EVENT_SPINS: usize = 64;
        for _ in 0..max_iterations {
            let dr = self.tui_host_dispatch_redraw_inner(line)?;
            let pn = self.tui_host_process_next_inner(PER_EVENT_SPINS, line)?;
            if self.take_tui_host_quit_requested() {
                break;
            }
            if dr == 0 && pn == 0 {
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
        tui.host = fpas_std::TuiHost::new();
        tui.on_key_pressed = None;
        tui.on_mouse = None;
        tui.on_paste = None;
        tui.on_focus_gained = None;
        tui.on_focus_lost = None;
        tui.on_resize = None;
        tui.on_paint = None;
        tui.on_idle = None;
        tui.idle_interval_ms = 0;
        tui.on_exit = None;
        tui.last_exit_reason = None;
        tui.quit_requested = false;
        tui.host_stop_requested = false;
        tui.run_active = false;
        tui.views.clear();
        close_result?;
        Ok(())
    }
}
