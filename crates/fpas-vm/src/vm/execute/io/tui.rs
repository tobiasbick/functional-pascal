//! `Std.Tui` VM execution helpers.
//!
//! **Documentation:** `docs/pascal/std/tui.md`, `docs/pascal/std/tui-app.md` (from the repository root).

use crate::vm::diagnostics::{TYPE_MISMATCH_CODE, VmError};
use crate::vm::{Worker, canonical_name, runtime_error};
use fpas_bytecode::{Intrinsic, SourceLocation, Value};
use fpas_diagnostics::codes::{
    RUNTIME_INTRINSIC_STACK_STATE_ERROR, RUNTIME_UNDEFINED_FUNCTION,
    RUNTIME_VM_OPERAND_TYPE_MISMATCH, RUNTIME_WRONG_CALL_ARITY,
};
use fpas_std::{ConsoleKeyEvent, HostEvent, TuiEvent};

const TUI_APPLICATION_TYPE: &str = "Std.Tui.Application";
const TUI_APPLICATION_HANDLERS_TYPE: &str = "Std.Tui.ApplicationHandlers";
const TUI_SIZE_TYPE: &str = "Std.Tui.Size";
const TUI_EVENT_TYPE: &str = "Std.Tui.TuiEvent";

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
                    tui.on_resize = None;
                    tui.on_paint = None;
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
                let event = {
                    let tui = self.shared.tui.lock().unwrap_or_else(|e| e.into_inner());
                    self.with_console_and_key_input(|console, key_input| {
                        tui.session
                            .read_event_timeout(console, key_input, timeout_ms, line)
                    })?
                };
                match event {
                    Some(event) => {
                        self.push(Value::OptionSome(Box::new(Self::tui_event_record(event))))?
                    }
                    None => self.push(Value::OptionNone)?,
                }
            }
            Intrinsic::TuiApplicationPollEvent => {
                self.pop_tui_application(line)?;
                let event = {
                    let tui = self.shared.tui.lock().unwrap_or_else(|e| e.into_inner());
                    self.with_console_and_key_input(|console, key_input| {
                        tui.session.poll_event(console, key_input, line)
                    })?
                };
                match event {
                    Some(event) => {
                        self.push(Value::OptionSome(Box::new(Self::tui_event_record(event))))?
                    }
                    None => self.push(Value::OptionNone)?,
                }
            }
            Intrinsic::TuiApplicationRequestRedraw => {
                self.pop_tui_application(line)?;
                let mut tui = self.shared.tui.lock().unwrap_or_else(|e| e.into_inner());
                tui.session.request_redraw(line)?;
            }
            Intrinsic::TuiApplicationRedrawPending => {
                self.pop_tui_application(line)?;
                let pending = {
                    let mut tui = self.shared.tui.lock().unwrap_or_else(|e| e.into_inner());
                    tui.session.take_redraw_pending(line)?
                };
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
                            tui.session.poll_event(console, key_input, line)
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
                        };
                        self.push(Value::OptionSome(Box::new(Self::tui_event_record(tui_ev))))?;
                    }
                }
            }
            Intrinsic::TuiHostRegisterOnKeyPressed => {
                let func = self.pop(line)?;
                self.pop_tui_application(line)?;
                self.validate_host_handler_function(
                    &func,
                    2,
                    "OnKeyPressed",
                    "Pass a `function (Application, Std.Console.KeyEvent): boolean`.",
                    line,
                )?;
                let mut tui = self.shared.tui.lock().unwrap_or_else(|e| e.into_inner());
                tui.on_key_pressed = Some(func);
            }
            Intrinsic::TuiHostRegisterOnResize => {
                let func = self.pop(line)?;
                self.pop_tui_application(line)?;
                self.validate_host_handler_function(
                    &func,
                    2,
                    "OnResize",
                    "Pass a `procedure (Application, Std.Tui.Size)` (two parameters).",
                    line,
                )?;
                let mut tui = self.shared.tui.lock().unwrap_or_else(|e| e.into_inner());
                tui.on_resize = Some(func);
            }
            Intrinsic::TuiHostProcessNext => {
                let max_spins = self.pop_int(line)?.max(0).min(4096) as usize;
                self.pop_tui_application(line)?;
                let tag = self.tui_host_process_next_inner(max_spins, line)?;
                self.push(Value::Integer(tag))?;
            }
            Intrinsic::TuiHostRegisterOnPaint => {
                let func = self.pop(line)?;
                self.pop_tui_application(line)?;
                self.validate_host_handler_function(
                    &func,
                    1,
                    "OnPaint",
                    "Pass a `procedure (Application)` (one parameter).",
                    line,
                )?;
                let mut tui = self.shared.tui.lock().unwrap_or_else(|e| e.into_inner());
                tui.on_paint = Some(func);
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
                let mut tui = self.shared.tui.lock().unwrap_or_else(|e| e.into_inner());
                tui.on_idle = Some(func);
                tui.idle_interval_ms = milliseconds;
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
                let mut tui = self.shared.tui.lock().unwrap_or_else(|e| e.into_inner());
                tui.quit_requested = true;
            }
            Intrinsic::TuiHostRegisterOnExit => {
                let func = self.pop(line)?;
                self.pop_tui_application(line)?;
                self.validate_host_handler_function(
                    &func,
                    2,
                    "OnExit",
                    "Pass a `procedure (Application, Std.Tui.ExitReason)` (two parameters).",
                    line,
                )?;
                let mut tui = self.shared.tui.lock().unwrap_or_else(|e| e.into_inner());
                tui.on_exit = Some(func);
            }
            Intrinsic::TuiHostRegisterOnMouse => {
                let func = self.pop(line)?;
                self.pop_tui_application(line)?;
                self.validate_host_handler_function(
                    &func,
                    2,
                    "OnMouse",
                    "Pass a `procedure (Application, Std.Console.Event)` (two parameters).",
                    line,
                )?;
                let mut tui = self.shared.tui.lock().unwrap_or_else(|e| e.into_inner());
                tui.on_mouse = Some(func);
            }
            Intrinsic::TuiHostInvokeOnKeyPressed => {
                let key_ev = self.pop_console_key_event(line)?;
                self.pop_tui_application(line)?;
                let handler = {
                    let tui = self.shared.tui.lock().unwrap_or_else(|e| e.into_inner());
                    tui.on_key_pressed.clone()
                };
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

    fn validate_host_handler_function(
        &self,
        func: &Value,
        arity: u8,
        label: &str,
        help: &'static str,
        line: SourceLocation,
    ) -> Result<(), VmError> {
        match func {
            Value::Function { name, .. } => {
                let (_, found_arity) = self
                    .shared
                    .chunk
                    .functions
                    .get(name.as_str())
                    .or_else(|| self.shared.chunk.functions.get(&canonical_name(name)))
                    .copied()
                    .ok_or_else(|| {
                        runtime_error(
                            RUNTIME_UNDEFINED_FUNCTION,
                            format!("Undefined function `{name}` for {label}"),
                            "Declare the handler before registering it.",
                            line,
                        )
                    })?;
                if found_arity != arity {
                    return Err(runtime_error(
                        RUNTIME_WRONG_CALL_ARITY,
                        format!("{label} handler must have arity {arity}, got {found_arity}"),
                        help,
                        line,
                    ));
                }
                Ok(())
            }
            _ => Err(runtime_error(
                RUNTIME_VM_OPERAND_TYPE_MISMATCH,
                format!("{label} expects a function value"),
                help,
                line,
            )),
        }
    }

    fn pop_tui_application_handlers(
        &mut self,
        line: SourceLocation,
    ) -> Result<Vec<(String, Value)>, VmError> {
        match self.pop(line)? {
            Value::Record { type_name, fields } if type_name == TUI_APPLICATION_HANDLERS_TYPE => {
                Ok(fields)
            }
            other => Err(runtime_error(
                TYPE_MISMATCH_CODE,
                format!(
                    "Expected {TUI_APPLICATION_HANDLERS_TYPE}, got {}",
                    other.type_name()
                ),
                "Pass a `Std.Tui.ApplicationHandlers` record to `Application.Configure(App, Handlers)`.",
                line,
            )),
        }
    }

    fn required_record_field<'a>(
        fields: &'a [(String, Value)],
        field_name: &str,
        line: SourceLocation,
    ) -> Result<&'a Value, VmError> {
        fields
            .iter()
            .find(|(name, _)| name.eq_ignore_ascii_case(field_name))
            .map(|(_, value)| value)
            .ok_or_else(|| {
                runtime_error(
                    RUNTIME_VM_OPERAND_TYPE_MISMATCH,
                    format!(
                        "Application.Configure(App, Handlers) is missing field `{field_name}`"
                    ),
                    format!(
                        "Build `ApplicationHandlers` with `{field_name} := ...`; malformed bytecode or a broken caller skipped that field."
                    ),
                    line,
                )
            })
    }

    fn integer_record_field(
        &self,
        fields: &[(String, Value)],
        field_name: &str,
        line: SourceLocation,
    ) -> Result<i64, VmError> {
        match Self::required_record_field(fields, field_name, line)? {
            Value::Integer(value) => Ok(*value),
            other => Err(runtime_error(
                TYPE_MISMATCH_CODE,
                format!(
                    "ApplicationHandlers.{field_name} must be integer, got {}",
                    other.type_name()
                ),
                format!(
                    "Set `{field_name} := <milliseconds>` with an integer value in the handler bundle."
                ),
                line,
            )),
        }
    }

    fn optional_host_handler_field(
        &self,
        fields: &[(String, Value)],
        field_name: &str,
        arity: u8,
        label: &str,
        help: &'static str,
        line: SourceLocation,
    ) -> Result<Option<Value>, VmError> {
        match Self::required_record_field(fields, field_name, line)? {
            Value::OptionNone => Ok(None),
            Value::OptionSome(inner) => {
                self.validate_host_handler_function(inner, arity, label, help, line)?;
                Ok(Some((**inner).clone()))
            }
            _ => Err(runtime_error(
                RUNTIME_VM_OPERAND_TYPE_MISMATCH,
                format!("ApplicationHandlers.{field_name} must be `Some(handler)` or `None`"),
                help,
                line,
            )),
        }
    }

    /// Returns status tag pushed by `TuiHostProcessNext` (`0`..=`4`).
    pub(super) fn tui_host_process_next_inner(
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
                tui.session.poll_event(console, key_input, line)
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

        let (on_key, on_mouse, on_resize) = {
            let tui = self.shared.tui.lock().unwrap_or_else(|e| e.into_inner());
            (
                tui.on_key_pressed.clone(),
                tui.on_mouse.clone(),
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

    /// `0` = no redraw pending, `5` = `OnPaint` ran, `6` = pending but no handler (cleared).
    pub(super) fn tui_host_dispatch_redraw_inner(
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

    /// One iteration: [`Self::tui_host_dispatch_redraw_inner`] then [`Self::tui_host_process_next_inner`].
    /// Stops when both return `0` (idle). `max_iterations` of `0` is a no-op.
    fn tui_host_run_loop_inner(
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

    /// Clears the flag when set so a later `TuiHostRunLoop` does not stop immediately.
    pub(super) fn take_tui_host_quit_requested(&self) -> bool {
        let mut tui = self.shared.tui.lock().unwrap_or_else(|e| e.into_inner());
        if tui.quit_requested {
            tui.quit_requested = false;
            true
        } else {
            false
        }
    }

    /// Converts `Application.Close(App)` into a structured host stop while `Application.Run` is active.
    pub(super) fn request_tui_host_stop_for_active_run(&self) -> bool {
        let mut tui = self.shared.tui.lock().unwrap_or_else(|e| e.into_inner());
        if tui.run_active {
            tui.host_stop_requested = true;
            true
        } else {
            false
        }
    }

    pub(super) fn pop_tui_application(&mut self, line: SourceLocation) -> Result<(), VmError> {
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

    pub(super) fn close_tui_application_state(
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
        tui.on_resize = None;
        tui.on_paint = None;
        tui.on_idle = None;
        tui.idle_interval_ms = 0;
        tui.on_exit = None;
        tui.last_exit_reason = None;
        tui.quit_requested = false;
        tui.host_stop_requested = false;
        tui.run_active = false;
        close_result?;
        Ok(())
    }

    pub(super) fn tui_application_record() -> Value {
        Value::Record {
            type_name: TUI_APPLICATION_TYPE.into(),
            fields: vec![],
        }
    }

    fn tui_size_record(width: i64, height: i64) -> Value {
        Value::Record {
            type_name: TUI_SIZE_TYPE.into(),
            fields: vec![
                ("width".into(), Value::Integer(width)),
                ("height".into(), Value::Integer(height)),
            ],
        }
    }

    fn tui_unknown_key_event() -> Value {
        Self::key_event_record(ConsoleKeyEvent::new(
            fpas_std::key_event::key_kind_index("Unknown"),
            '\0',
            false,
            false,
            false,
            false,
        ))
    }

    fn tui_event_record(event: TuiEvent) -> Value {
        match event {
            TuiEvent::Key(key) => Value::Record {
                type_name: TUI_EVENT_TYPE.into(),
                fields: vec![
                    ("kind".into(), Value::Integer(0)),
                    ("key".into(), Self::key_event_record(key)),
                    ("size".into(), Self::tui_size_record(0, 0)),
                ],
            },
            TuiEvent::Resize { width, height } => Value::Record {
                type_name: TUI_EVENT_TYPE.into(),
                fields: vec![
                    ("kind".into(), Value::Integer(1)),
                    ("key".into(), Self::tui_unknown_key_event()),
                    ("size".into(), Self::tui_size_record(width, height)),
                ],
            },
            TuiEvent::Mouse(_) => Value::Record {
                type_name: TUI_EVENT_TYPE.into(),
                fields: vec![
                    ("kind".into(), Value::Integer(2)),
                    ("key".into(), Self::tui_unknown_key_event()),
                    ("size".into(), Self::tui_size_record(0, 0)),
                ],
            },
        }
    }
}
