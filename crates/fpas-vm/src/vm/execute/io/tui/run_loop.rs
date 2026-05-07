//! `Std.Tui` run loop, event dispatch, and session lifecycle.
//!
//! **Documentation:** `docs/pascal/std/tui-app.md` (from the repository root).

use crate::vm::diagnostics::{TYPE_MISMATCH_CODE, VmError};
use crate::vm::{Worker, runtime_error};
use fpas_bytecode::{SourceLocation, Value};
use fpas_std::{CommandId, HostEvent};

const TUI_APPLICATION_TYPE: &str = "Std.Tui.Application";
/// Discriminant of `Std.Console.KeyKind.Tab`; must match
/// [`fpas_std::key_event::KEY_KIND_VARIANTS`] (index 2).
const KEY_KIND_TAB: usize = 2;
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
                // Phase 7 Step 2: Tab / Shift+Tab trigger host-managed focus traversal when
                // the focus chain has children.  The key is consumed and never reaches
                // OnKeyPressed in that case.
                if k.kind == KEY_KIND_TAB {
                    let (changed, had_previous) = {
                        let mut tui = self.shared.tui.lock().unwrap_or_else(|e| e.into_inner());
                        if k.shift {
                            tui.views.focus_prev()
                        } else {
                            tui.views.focus_next()
                        }
                    };
                    if changed {
                        self.invoke_focus_transition(had_previous, line)?;
                        return Ok(if k.shift { 15 } else { 14 });
                    }
                    // No focusable children or single-element chain already focused:
                    // fall through to normal OnKeyPressed dispatch.
                }
                if let Some(command_id) = self.resolve_tui_command(&k) {
                    return self.dispatch_tui_command(command_id, line);
                }
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
            HostEvent::Mouse(ev) => self.dispatch_console_event_handler(
                on_mouse,
                app_rec,
                Self::console_event_record(ev),
                5,
                7,
                line,
            ),
            HostEvent::Paste(ev) => self.dispatch_console_event_handler(
                on_paste,
                app_rec,
                Self::console_event_record(ev),
                8,
                9,
                line,
            ),
            HostEvent::FocusGained(ev) => self.dispatch_console_event_handler(
                on_focus_gained,
                app_rec,
                Self::console_event_record(ev),
                10,
                11,
                line,
            ),
            HostEvent::FocusLost(ev) => self.dispatch_console_event_handler(
                on_focus_lost,
                app_rec,
                Self::console_event_record(ev),
                12,
                13,
                line,
            ),
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

    /// Dispatches a `Std.Console.Event`-bearing handler.
    ///
    /// Returns `hit_tag` when `handler` is present (and calls it), `miss_tag` otherwise.
    fn dispatch_console_event_handler(
        &mut self,
        handler: Option<Value>,
        app_rec: Value,
        event_rec: Value,
        hit_tag: i64,
        miss_tag: i64,
        line: SourceLocation,
    ) -> Result<i64, VmError> {
        if let Some(h) = handler {
            let _ = self.call_function_sync(&h, &[app_rec, event_rec], line)?;
            Ok(hit_tag)
        } else {
            Ok(miss_tag)
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

    fn resolve_tui_command(&self, key: &fpas_std::ConsoleKeyEvent) -> Option<CommandId> {
        let tui = self.shared.tui.lock().unwrap_or_else(|e| e.into_inner());
        tui.commands.resolve(key)
    }

    fn dispatch_tui_command(
        &mut self,
        command_id: CommandId,
        line: SourceLocation,
    ) -> Result<i64, VmError> {
        let handler = {
            let tui = self.shared.tui.lock().unwrap_or_else(|e| e.into_inner());
            tui.on_command.clone()
        };
        let Some(handler) = handler else {
            return Ok(17);
        };

        let app_rec = Self::tui_application_record();
        let _ =
            self.call_function_sync(&handler, &[app_rec, Value::Integer(command_id.0)], line)?;
        Ok(16)
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

    /// Fires `OnDeactivate` (if `fire_deactivate` is `true` and a handler is registered)
    /// then `OnActivate` (if registered) after a focus transition.
    ///
    /// Both handlers have the signature `procedure (Application)`.
    fn invoke_focus_transition(
        &mut self,
        fire_deactivate: bool,
        line: SourceLocation,
    ) -> Result<(), VmError> {
        let app_rec = Self::tui_application_record();

        if fire_deactivate {
            let handler = {
                let tui = self.shared.tui.lock().unwrap_or_else(|e| e.into_inner());
                tui.on_deactivate.clone()
            };
            if let Some(handler) = handler {
                let _ = self.call_function_sync(&handler, &[app_rec.clone()], line)?;
            }
        }

        let handler = {
            let tui = self.shared.tui.lock().unwrap_or_else(|e| e.into_inner());
            tui.on_activate.clone()
        };
        if let Some(handler) = handler {
            let _ = self.call_function_sync(&handler, &[app_rec], line)?;
        }

        Ok(())
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
        tui.on_activate = None;
        tui.on_deactivate = None;
        tui.on_command = None;
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
        tui.commands.clear();
        tui.modals.clear();
        close_result?;
        Ok(())
    }
}
