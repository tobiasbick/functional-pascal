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
const TUI_SIZE_TYPE: &str = "Std.Tui.Size";
const TUI_EVENT_TYPE: &str = "Std.Tui.TuiEvent";

impl Worker {
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
                }
                self.push(Self::tui_application_record())?;
            }
            Intrinsic::TuiApplicationClose => {
                self.pop_tui_application(line)?;
                let mut tui = self.shared.tui.lock().unwrap_or_else(|e| e.into_inner());
                self.with_console_and_key_input(|console, key_input| {
                    tui.session.close(console, key_input, line)
                })?;
                tui.host = fpas_std::TuiHost::new();
                tui.on_key_pressed = None;
                tui.on_resize = None;
                tui.on_paint = None;
                tui.quit_requested = false;
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

    /// Returns status tag pushed by `TuiHostProcessNext` (`0`..=`4`).
    fn tui_host_process_next_inner(
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

        let (on_key, on_resize) = {
            let tui = self.shared.tui.lock().unwrap_or_else(|e| e.into_inner());
            (tui.on_key_pressed.clone(), tui.on_resize.clone())
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
    fn tui_host_dispatch_redraw_inner(&mut self, line: SourceLocation) -> Result<i64, VmError> {
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
    fn take_tui_host_quit_requested(&self) -> bool {
        let mut tui = self.shared.tui.lock().unwrap_or_else(|e| e.into_inner());
        if tui.quit_requested {
            tui.quit_requested = false;
            true
        } else {
            false
        }
    }

    fn pop_tui_application(&mut self, line: SourceLocation) -> Result<(), VmError> {
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

    fn tui_application_record() -> Value {
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
        }
    }
}
