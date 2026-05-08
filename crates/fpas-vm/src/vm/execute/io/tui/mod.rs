//! `Std.Tui` VM execution — intrinsic dispatch.
//!
//! **Documentation:** `docs/pascal/std/tui.md`, `docs/pascal/std/tui-app.md` (from the repository root).

mod handlers;
mod records;
mod run_loop;

use crate::vm::diagnostics::VmError;
use crate::vm::{Worker, runtime_error};
use fpas_bytecode::{Intrinsic, SourceLocation, Value};
use fpas_diagnostics::codes::RUNTIME_INTRINSIC_STACK_STATE_ERROR;
use fpas_std::{HostEvent, TuiEvent, ViewId, ViewRect};

impl Worker {
    /// Execute a `Std.Tui` intrinsic in the VM.
    pub(super) fn try_exec_tui_intrinsic(
        &mut self,
        intrinsic: Intrinsic,
        line: SourceLocation,
    ) -> Result<bool, VmError> {
        match intrinsic {
            Intrinsic::TuiApplicationOpen => {
                {
                    let mut tui = self.shared.tui.lock().unwrap_or_else(|e| e.into_inner());
                    self.with_console_and_key_input(|console, key_input| {
                        tui.session.open(console, key_input, line)
                    })?;
                    tui.host = fpas_std::TuiHost::new();
                    tui.quit_requested = false;
                    tui.host_stop_requested = false;
                    tui.on_idle = None;
                    tui.idle_interval_ms = 0;
                    tui.on_exit = None;
                    tui.last_exit_reason = None;
                    tui.run_active = false;
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
                    tui.commands.clear();
                    tui.modals.clear();
                }
                self.push(Self::tui_application_record())?;
            }
            Intrinsic::TuiApplicationClose => {
                self.pop_tui_application(line)?;
                if !self.request_tui_host_stop_for_active_run() {
                    self.close_tui_application_state(line)?;
                }
            }
            Intrinsic::TuiApplicationConfigure => {
                let handlers = self.pop_tui_application_handlers(line)?;
                self.pop_tui_application(line)?;

                let on_paint = Self::required_record_field(&handlers, "OnPaint", line)?.clone();
                self.validate_host_handler_function(
                    &on_paint,
                    1,
                    "OnPaint",
                    "Set `OnPaint := Handler` where `Handler` is `procedure (Application)`.",
                    line,
                )?;
                let on_key_pressed = self.optional_host_handler_field(
                    &handlers,
                    "OnKeyPressed",
                    2,
                    "OnKeyPressed",
                    "Set `OnKeyPressed := Some(Handler)` or `None`; the handler must be `function (Application, Std.Console.KeyEvent): boolean`.",
                    line,
                )?;
                let on_mouse = self.optional_host_handler_field(
                    &handlers,
                    "OnMouse",
                    2,
                    "OnMouse",
                    "Set `OnMouse := Some(Handler)` or `None`; the handler must be `procedure (Application, Std.Console.Event)`.",
                    line,
                )?;
                let on_paste = self.optional_host_handler_field(
                    &handlers,
                    "OnPaste",
                    2,
                    "OnPaste",
                    "Set `OnPaste := Some(Handler)` or `None`; the handler must be `procedure (Application, Std.Console.Event)`.",
                    line,
                )?;
                let on_focus_gained = self.optional_host_handler_field(
                    &handlers,
                    "OnFocusGained",
                    2,
                    "OnFocusGained",
                    "Set `OnFocusGained := Some(Handler)` or `None`; the handler must be `procedure (Application, Std.Console.Event)`.",
                    line,
                )?;
                let on_focus_lost = self.optional_host_handler_field(
                    &handlers,
                    "OnFocusLost",
                    2,
                    "OnFocusLost",
                    "Set `OnFocusLost := Some(Handler)` or `None`; the handler must be `procedure (Application, Std.Console.Event)`.",
                    line,
                )?;
                let on_activate = self.optional_host_handler_field(
                    &handlers,
                    "OnActivate",
                    1,
                    "OnActivate",
                    "Set `OnActivate := Some(Handler)` or `None`; the handler must be `procedure (Application)`.",
                    line,
                )?;
                let on_deactivate = self.optional_host_handler_field(
                    &handlers,
                    "OnDeactivate",
                    1,
                    "OnDeactivate",
                    "Set `OnDeactivate := Some(Handler)` or `None`; the handler must be `procedure (Application)`.",
                    line,
                )?;
                let on_command = self.optional_host_handler_field(
                    &handlers,
                    "OnCommand",
                    2,
                    "OnCommand",
                    "Set `OnCommand := Some(Handler)` or `None`; the handler must be `procedure (Application, integer)`.",
                    line,
                )?;
                let on_resize = self.optional_host_handler_field(
                    &handlers,
                    "OnResize",
                    2,
                    "OnResize",
                    "Set `OnResize := Some(Handler)` or `None`; the handler must be `procedure (Application, Std.Tui.Size)`.",
                    line,
                )?;
                let idle_interval_ms = self
                    .integer_record_field(&handlers, "OnIdleMilliseconds", line)?
                    .max(0);
                let on_idle = self.optional_host_handler_field(
                    &handlers,
                    "OnIdle",
                    1,
                    "OnIdle",
                    "Set `OnIdle := Some(Handler)` or `None`; the handler must be `procedure (Application)`.",
                    line,
                )?;
                let on_exit = self.optional_host_handler_field(
                    &handlers,
                    "OnExit",
                    2,
                    "OnExit",
                    "Set `OnExit := Some(Handler)` or `None`; the handler must be `procedure (Application, Std.Tui.ExitReason)`.",
                    line,
                )?;

                let mut tui = self.shared.tui.lock().unwrap_or_else(|e| e.into_inner());
                tui.on_paint = Some(on_paint);
                tui.on_key_pressed = on_key_pressed;
                tui.on_mouse = on_mouse;
                tui.on_paste = on_paste;
                tui.on_focus_gained = on_focus_gained;
                tui.on_focus_lost = on_focus_lost;
                tui.on_activate = on_activate;
                tui.on_deactivate = on_deactivate;
                tui.on_command = on_command;
                tui.on_resize = on_resize;
                tui.idle_interval_ms = idle_interval_ms;
                tui.on_idle = on_idle;
                tui.on_exit = on_exit;
            }
            Intrinsic::TuiApplicationRun => {
                self.tui_application_run(line)?;
                self.push(Value::Unit)?;
            }
            Intrinsic::TuiApplicationSize => {
                self.pop_tui_application(line)?;
                let (width, height) = {
                    let tui = self.shared.tui.lock().unwrap_or_else(|e| e.into_inner());
                    self.with_console(|console| tui.session.size(console, line))?
                };
                self.push(Self::tui_size_record(width, height))?;
            }
            Intrinsic::TuiApplicationReadEvent => {
                self.pop_tui_application(line)?;
                let event = {
                    let tui = self.shared.tui.lock().unwrap_or_else(|e| e.into_inner());
                    self.with_console_and_key_input(|console, key_input| {
                        tui.session.read_event(console, key_input, line)
                    })?
                };
                self.push(Self::tui_event_record(event))?;
            }
            Intrinsic::TuiApplicationReadEventTimeout => {
                let timeout_ms = self.pop_int(line)?;
                self.pop_tui_application(line)?;
                loop {
                    let event = {
                        let tui = self.shared.tui.lock().unwrap_or_else(|e| e.into_inner());
                        self.with_console_and_key_input(|console, key_input| {
                            tui.session
                                .read_event_timeout(console, key_input, timeout_ms, line)
                        })?
                    };
                    match event {
                        None => {
                            self.push(Value::OptionNone)?;
                            break;
                        }
                        // Paste and focus events are dispatch-only; skip them in the read path.
                        Some(
                            TuiEvent::Paste(_) | TuiEvent::FocusGained(_) | TuiEvent::FocusLost(_),
                        ) => continue,
                        Some(event) => {
                            self.push(Value::OptionSome(Box::new(Self::tui_event_record(event))))?;
                            break;
                        }
                    }
                }
            }
            Intrinsic::TuiApplicationPollEvent => {
                self.pop_tui_application(line)?;
                loop {
                    let event = {
                        let tui = self.shared.tui.lock().unwrap_or_else(|e| e.into_inner());
                        self.with_console_and_key_input(|console, key_input| {
                            tui.session.poll_event(console, key_input, line)
                        })?
                    };
                    match event {
                        None => {
                            self.push(Value::OptionNone)?;
                            break;
                        }
                        // Paste and focus events are dispatch-only; skip them in the poll path.
                        Some(
                            TuiEvent::Paste(_) | TuiEvent::FocusGained(_) | TuiEvent::FocusLost(_),
                        ) => continue,
                        Some(event) => {
                            self.push(Value::OptionSome(Box::new(Self::tui_event_record(event))))?;
                            break;
                        }
                    }
                }
            }
            Intrinsic::TuiApplicationRequestRedraw => {
                self.pop_tui_application(line)?;
                self.with_tui(|tui| tui.session.request_redraw(line))?;
            }
            Intrinsic::TuiApplicationRedrawPending => {
                self.pop_tui_application(line)?;
                let pending = self.with_tui(|tui| tui.session.take_redraw_pending(line))?;
                self.push(Value::Boolean(pending))?;
            }
            Intrinsic::TuiHostPollNext => {
                self.pop_tui_application(line)?;
                let mapped = {
                    let mut tui = self.shared.tui.lock().unwrap_or_else(|e| e.into_inner());
                    if let Some(ev) = tui.host.pop_ready_event() {
                        Some(ev)
                    } else {
                        let polled = self.with_console_and_key_input(|console, key_input| {
                            tui.session.poll_event_all(console, key_input, line)
                        })?;
                        if let Some(tui_ev) = polled {
                            tui.host.ingest_tui_event(tui_ev);
                        }
                        tui.host.pop_ready_event()
                    }
                };
                match mapped {
                    None => self.push(Value::OptionNone)?,
                    Some(ev) => {
                        let tui_ev = match ev {
                            HostEvent::Resize { width, height } => {
                                TuiEvent::Resize { width, height }
                            }
                            HostEvent::Key(k) => TuiEvent::Key(k),
                            HostEvent::Mouse(m) => TuiEvent::Mouse(m),
                            // Paste and focus events are dispatch-only; not exposed via poll API.
                            HostEvent::Paste(_)
                            | HostEvent::FocusGained(_)
                            | HostEvent::FocusLost(_) => {
                                self.push(Value::OptionNone)?;
                                return Ok(true);
                            }
                        };
                        self.push(Value::OptionSome(Box::new(Self::tui_event_record(tui_ev))))?;
                    }
                }
            }
            Intrinsic::TuiHostRegisterOnKeyPressed => {
                self.register_tui_handler(
                    2,
                    "OnKeyPressed",
                    "Pass a `function (Application, Std.Console.KeyEvent): boolean`.",
                    |tui, f| tui.on_key_pressed = Some(f),
                    line,
                )?;
            }
            Intrinsic::TuiHostRegisterOnResize => {
                self.register_tui_handler(
                    2,
                    "OnResize",
                    "Pass a `procedure (Application, Std.Tui.Size)` (two parameters).",
                    |tui, f| tui.on_resize = Some(f),
                    line,
                )?;
            }
            Intrinsic::TuiHostProcessNext => {
                let max_spins = self.pop_int(line)?.max(0).min(4096) as usize;
                self.pop_tui_application(line)?;
                let tag = self.tui_host_process_next_inner(max_spins, line)?;
                self.push(Value::Integer(tag))?;
            }
            Intrinsic::TuiHostRegisterOnPaint => {
                self.register_tui_handler(
                    1,
                    "OnPaint",
                    "Pass a `procedure (Application)` (one parameter).",
                    |tui, f| tui.on_paint = Some(f),
                    line,
                )?;
            }
            Intrinsic::TuiHostRegisterOnIdle => {
                let func = self.pop(line)?;
                let milliseconds = self.pop_int(line)?.max(0);
                self.pop_tui_application(line)?;
                self.validate_host_handler_function(
                    &func,
                    1,
                    "OnIdle",
                    "Pass `Application`, an idle interval in milliseconds, and a `procedure (Application)` handler.",
                    line,
                )?;
                self.with_tui(|tui| {
                    tui.on_idle = Some(func);
                    tui.idle_interval_ms = milliseconds;
                });
            }
            Intrinsic::TuiHostDispatchRedraw => {
                self.pop_tui_application(line)?;
                let tag = self.tui_host_dispatch_redraw_inner(line)?;
                self.push(Value::Integer(tag))?;
            }
            Intrinsic::TuiHostRunLoop => {
                let max_iters = self.pop_int(line)?.max(0).min(1_000_000) as usize;
                self.pop_tui_application(line)?;
                self.tui_host_run_loop_inner(max_iters, line)?;
                self.push(Value::Unit)?;
            }
            Intrinsic::TuiHostRequestQuit => {
                self.pop_tui_application(line)?;
                self.with_tui(|tui| tui.quit_requested = true);
            }
            Intrinsic::TuiHostRegisterOnExit => {
                self.register_tui_handler(
                    2,
                    "OnExit",
                    "Pass a `procedure (Application, Std.Tui.ExitReason)` (two parameters).",
                    |tui, f| tui.on_exit = Some(f),
                    line,
                )?;
            }
            Intrinsic::TuiHostRegisterOnMouse => {
                self.register_tui_handler(
                    2,
                    "OnMouse",
                    "Pass a `procedure (Application, Std.Console.Event)` (two parameters).",
                    |tui, f| tui.on_mouse = Some(f),
                    line,
                )?;
            }
            Intrinsic::TuiHostRegisterOnPaste => {
                self.register_tui_handler(
                    2,
                    "OnPaste",
                    "Pass a `procedure (Application, Std.Console.Event)` (two parameters).",
                    |tui, f| tui.on_paste = Some(f),
                    line,
                )?;
            }
            Intrinsic::TuiHostRegisterOnFocusGained => {
                self.register_tui_handler(
                    2,
                    "OnFocusGained",
                    "Pass a `procedure (Application, Std.Console.Event)` (two parameters).",
                    |tui, f| tui.on_focus_gained = Some(f),
                    line,
                )?;
            }
            Intrinsic::TuiHostRegisterOnFocusLost => {
                self.register_tui_handler(
                    2,
                    "OnFocusLost",
                    "Pass a `procedure (Application, Std.Console.Event)` (two parameters).",
                    |tui, f| tui.on_focus_lost = Some(f),
                    line,
                )?;
            }
            Intrinsic::TuiHostRegisterOnActivate => {
                self.register_tui_handler(
                    1,
                    "OnActivate",
                    "Pass a `procedure (Application)` (one parameter).",
                    |tui, f| tui.on_activate = Some(f),
                    line,
                )?;
            }
            Intrinsic::TuiHostRegisterOnDeactivate => {
                self.register_tui_handler(
                    1,
                    "OnDeactivate",
                    "Pass a `procedure (Application)` (one parameter).",
                    |tui, f| tui.on_deactivate = Some(f),
                    line,
                )?;
            }
            Intrinsic::TuiHostRegisterOnCommand => {
                self.register_tui_handler(
                    2,
                    "OnCommand",
                    "Pass a `procedure (Application, integer)` (two parameters).",
                    |tui, f| tui.on_command = Some(f),
                    line,
                )?;
            }
            Intrinsic::TuiHostBindCommand => {
                let command_id = self.pop_int(line)?;
                let key = self.pop_console_key_event(line)?;
                self.pop_tui_application(line)?;
                self.with_tui(|tui| {
                    tui.commands.bind(key, fpas_std::CommandId(command_id));
                });
            }
            Intrinsic::TuiHostEnterModal => {
                let modal_id = self.pop_int(line)?;
                self.pop_tui_application(line)?;
                self.with_tui(|tui| {
                    tui.modals.enter(fpas_std::ModalId(modal_id));
                });
            }
            Intrinsic::TuiHostLeaveModal => {
                self.pop_tui_application(line)?;
                self.with_tui(|tui| {
                    let popped_views = tui
                        .modals
                        .leave_with_scoped_views()
                        .map(|(_, scoped_views)| scoped_views)
                        .unwrap_or_default();
                    for view_id in popped_views {
                        if let Some(rect) = tui.views.rect(view_id) {
                            let _ = tui.session.request_redraw_rect(rect, line);
                        }
                    }
                    let revealed_views = tui
                        .modals
                        .active_scoped_views()
                        .map(|views| views.to_vec())
                        .unwrap_or_default();
                    for view_id in revealed_views {
                        if let Some(rect) = tui.views.rect(view_id) {
                            let _ = tui.session.request_redraw_rect(rect, line);
                        }
                    }
                });
            }
            Intrinsic::TuiHostModalDepth => {
                self.pop_tui_application(line)?;
                let depth = self.with_tui(|tui| tui.modals.depth() as i64);
                self.push(Value::Integer(depth))?;
            }
            Intrinsic::TuiHostRegisterView => {
                let height = self.pop_int(line)?;
                let width = self.pop_int(line)?;
                let y = self.pop_int(line)?;
                let x = self.pop_int(line)?;
                self.pop_tui_application(line)?;
                let view_rect = ViewRect {
                    x,
                    y,
                    width,
                    height,
                };
                let view_id = self.with_tui(|tui| {
                    let view_id = tui.views.register(view_rect);
                    let _ = tui.session.request_redraw_rect(view_rect, line);
                    view_id
                });
                self.push(Value::Integer(i64::from(view_id.raw())))?;
            }
            Intrinsic::TuiHostUnregisterView => {
                let view_id = self.pop_tui_view_id(line)?;
                self.pop_tui_application(line)?;
                self.with_tui(|tui| {
                    let previous_focus = tui.views.focused_id();
                    if let Some(rect) = tui.views.rect(view_id) {
                        let _ = tui.session.request_redraw_rect(rect, line);
                    }
                    tui.views.unregister(view_id);
                    let current_focus = tui.views.focused_id();
                    if current_focus != previous_focus
                        && let Some(view_id) = current_focus
                        && let Some(rect) = tui.views.rect(view_id)
                    {
                        let _ = tui.session.request_redraw_rect(rect, line);
                    }
                });
            }
            Intrinsic::TuiHostPushChildView => {
                let view_id = self.pop_tui_view_id(line)?;
                self.pop_tui_application(line)?;
                self.with_tui(|tui| {
                    tui.views.push_child(view_id);
                });
            }
            Intrinsic::TuiHostQueryFocusedViewId => {
                self.pop_tui_application(line)?;
                let focused_id = self.with_tui(|tui| tui.views.focused_id());
                let packed = focused_id.map_or(-1, |id| i64::from(id.raw()));
                self.push(Value::Integer(packed))?;
            }
            Intrinsic::TuiHostAttachViewToActiveModal => {
                let view_id = self.pop_tui_view_id(line)?;
                self.pop_tui_application(line)?;
                self.with_tui(|tui| {
                    if tui.modals.attach_view_to_active(view_id)
                        && let Some(rect) = tui.views.rect(view_id)
                    {
                        let _ = tui.session.request_redraw_rect(rect, line);
                    }
                });
            }
            Intrinsic::TuiHostSetViewRect => {
                let height = self.pop_int(line)?;
                let width = self.pop_int(line)?;
                let y = self.pop_int(line)?;
                let x = self.pop_int(line)?;
                let view_id = self.pop_tui_view_id(line)?;
                self.pop_tui_application(line)?;
                self.with_tui(|tui| {
                    let Some(previous_rect) = tui.views.rect(view_id) else {
                        return;
                    };
                    let next_rect = ViewRect {
                        x,
                        y,
                        width,
                        height,
                    };
                    tui.views.set_rect(view_id, next_rect);
                    let _ = tui.session.request_redraw_rect(previous_rect, line);
                    let _ = tui.session.request_redraw_rect(next_rect, line);
                });
            }
            Intrinsic::TuiHostInvokeOnKeyPressed => {
                let key_ev = self.pop_console_key_event(line)?;
                self.pop_tui_application(line)?;
                let handler = self.with_tui(|tui| tui.on_key_pressed.clone());
                let handler = handler.ok_or_else(|| {
                    runtime_error(
                        RUNTIME_INTRINSIC_STACK_STATE_ERROR,
                        "No OnKeyPressed handler is registered for the Tui host",
                        "Call `TuiHostRegisterOnKeyPressed` after `Application.Open` with a `function (Application, Std.Console.KeyEvent): boolean`.",
                        line,
                    )
                })?;
                let app_rec = Self::tui_application_record();
                let consumed = self.call_function_sync(
                    &handler,
                    &[app_rec, Self::key_event_record(key_ev)],
                    line,
                )?;
                self.push(consumed)?;
            }
            _ => return Ok(false),
        }

        Ok(true)
    }

    fn pop_tui_view_id(&mut self, line: SourceLocation) -> Result<ViewId, VmError> {
        let raw = self.pop_int(line)?;
        let raw = u32::try_from(raw).map_err(|_| {
            runtime_error(
                RUNTIME_INTRINSIC_STACK_STATE_ERROR,
                format!("ViewId {raw} is out of range (expected 0..={})", u32::MAX),
                "Pass the integer handle returned by `Application.HostRegisterView(App, X, Y, Width, Height)`.",
                line,
            )
        })?;
        Ok(ViewId::from_raw(raw))
    }
}
