//! Hosted graph callbacks and deterministic event dispatch.

use fpas_bytecode::{GraphIntrinsic, SourceLocation, Value};
use fpas_diagnostics::codes::{
    RUNTIME_INTRINSIC_STACK_STATE_ERROR, RUNTIME_VM_OPERAND_TYPE_MISMATCH,
};
use fpas_std::{GraphEvent, UiEvent, UiMouse, UiResize, UiWheel};

use crate::vm::execute::io::console_records::key_event_record;
use crate::vm::register::VmError;
use crate::vm::register::worker::RegisterWorker;

use super::{application, integer, records};

impl RegisterWorker {
    pub(super) fn execute_graph_host(
        &self,
        operation: GraphIntrinsic,
        arguments: &[Value],
        location: SourceLocation,
    ) -> Result<Option<Option<Value>>, VmError> {
        let result = match operation {
            GraphIntrinsic::ApplicationConfigure => {
                application(arguments.first(), self)?;
                let handlers =
                    record_fields(arguments.get(1), "Std.Graph.ApplicationHandlers", self)?;
                let on_paint = required(&handlers, "OnPaint", self)?.clone();
                let on_key_pressed = optional(&handlers, "OnKeyPressed", self)?;
                let on_mouse = optional(&handlers, "OnMouse", self)?;
                let on_wheel = optional(&handlers, "OnWheel", self)?;
                let on_resize = optional(&handlers, "OnResize", self)?;
                let on_close_requested = optional(&handlers, "OnCloseRequested", self)?;
                let on_idle = optional(&handlers, "OnIdle", self)?;
                let on_exit = optional(&handlers, "OnExit", self)?;
                let idle_interval_ms = match required(&handlers, "OnIdleMilliseconds", self)? {
                    Value::Integer(value) => (*value).max(0),
                    actual => return Err(self.graph_host_type_error("integer", actual)),
                };
                self.with_graph(|graph| {
                    graph.on_paint = Some(on_paint);
                    graph.on_key_pressed = on_key_pressed;
                    graph.on_mouse = on_mouse;
                    graph.on_wheel = on_wheel;
                    graph.on_resize = on_resize;
                    graph.on_close_requested = on_close_requested;
                    graph.on_idle = on_idle;
                    graph.on_exit = on_exit;
                    graph.idle_interval_ms = idle_interval_ms;
                });
                None
            }
            GraphIntrinsic::ApplicationRun => {
                application(arguments.first(), self)?;
                self.run_graph_application(location)?;
                Some(Value::Unit)
            }
            GraphIntrinsic::HostRequestQuit => {
                application(arguments.first(), self)?;
                self.with_graph(|graph| graph.quit_requested = true);
                None
            }
            GraphIntrinsic::HostRegisterOnKeyPressed
            | GraphIntrinsic::HostRegisterOnResize
            | GraphIntrinsic::HostRegisterOnPaint
            | GraphIntrinsic::HostRegisterOnExit
            | GraphIntrinsic::HostRegisterOnMouse
            | GraphIntrinsic::HostRegisterOnWheel
            | GraphIntrinsic::HostRegisterOnCloseRequested => {
                application(arguments.first(), self)?;
                let callback = arguments
                    .get(1)
                    .ok_or_else(|| self.graph_host_arity_error())?
                    .clone();
                self.with_graph(|graph| match operation {
                    GraphIntrinsic::HostRegisterOnKeyPressed => {
                        graph.on_key_pressed = Some(callback)
                    }
                    GraphIntrinsic::HostRegisterOnResize => graph.on_resize = Some(callback),
                    GraphIntrinsic::HostRegisterOnPaint => graph.on_paint = Some(callback),
                    GraphIntrinsic::HostRegisterOnExit => graph.on_exit = Some(callback),
                    GraphIntrinsic::HostRegisterOnMouse => graph.on_mouse = Some(callback),
                    GraphIntrinsic::HostRegisterOnWheel => graph.on_wheel = Some(callback),
                    _ => graph.on_close_requested = Some(callback),
                });
                None
            }
            GraphIntrinsic::HostRegisterOnIdle => {
                application(arguments.first(), self)?;
                let milliseconds = integer(arguments, 1, 3, self)?.max(0);
                let callback = arguments
                    .get(2)
                    .ok_or_else(|| self.graph_host_arity_error())?
                    .clone();
                self.with_graph(|graph| {
                    graph.idle_interval_ms = milliseconds;
                    graph.on_idle = Some(callback);
                });
                None
            }
            GraphIntrinsic::HostProcessNext => {
                application(arguments.first(), self)?;
                let spins = integer(arguments, 1, 2, self)?.clamp(0, 4096) as usize;
                Some(Value::Integer(self.process_graph_event(spins, location)?))
            }
            GraphIntrinsic::HostDispatchRedraw => {
                application(arguments.first(), self)?;
                Some(Value::Integer(self.dispatch_graph_redraw(location)?))
            }
            _ => return Ok(None),
        };
        Ok(Some(result))
    }

    fn process_graph_event(&self, spins: usize, location: SourceLocation) -> Result<i64, VmError> {
        let mut ready = None;
        for _ in 0..spins.max(1) {
            let event = self.with_graph(|graph| {
                if let Some(event) = graph.host.pop_ready_event() {
                    return Ok(Some(event));
                }
                if let Some(event) = graph.session.read_host_ui_event_timeout(0, location)? {
                    graph.host.ingest_ui_event(event);
                }
                Ok(graph.host.pop_ready_event())
            })?;
            if event.is_some() {
                ready = event;
                break;
            }
        }
        let Some(event) = ready else { return Ok(0) };
        let app = records::application();
        match event {
            UiEvent::Key(key) => {
                let callback = self.with_graph(|graph| graph.on_key_pressed.clone());
                if let Some(callback) = callback {
                    self.call_callback_sync(&callback, vec![app, key_event_record(key)])?;
                    Ok(1)
                } else {
                    Ok(3)
                }
            }
            UiEvent::Resize(UiResize { width, height, .. }) => {
                self.with_graph(|graph| graph.session.request_redraw(location))?;
                let callback = self.with_graph(|graph| graph.on_resize.clone());
                if let Some(callback) = callback {
                    self.call_callback_sync(&callback, vec![app, records::size(width, height)])?;
                    Ok(2)
                } else {
                    Ok(4)
                }
            }
            UiEvent::Mouse(UiMouse {
                action,
                button,
                x,
                y,
                modifiers,
            }) => {
                let callback = self.with_graph(|graph| graph.on_mouse.clone());
                if let Some(callback) = callback {
                    self.call_callback_sync(
                        &callback,
                        vec![
                            app,
                            records::event(GraphEvent::Mouse {
                                action,
                                button,
                                x,
                                y,
                                shift: modifiers.shift,
                                ctrl: modifiers.ctrl,
                                alt: modifiers.alt,
                                meta: modifiers.meta,
                            }),
                        ],
                    )?;
                    Ok(5)
                } else {
                    Ok(7)
                }
            }
            UiEvent::Wheel(UiWheel {
                delta_x,
                delta_y,
                x,
                y,
                modifiers,
            }) => {
                let callback = self.with_graph(|graph| graph.on_wheel.clone());
                if let Some(callback) = callback {
                    self.call_callback_sync(
                        &callback,
                        vec![
                            app,
                            records::event(GraphEvent::Wheel {
                                delta_x,
                                delta_y,
                                x,
                                y,
                                shift: modifiers.shift,
                                ctrl: modifiers.ctrl,
                                alt: modifiers.alt,
                                meta: modifiers.meta,
                            }),
                        ],
                    )?;
                    Ok(8)
                } else {
                    Ok(9)
                }
            }
            UiEvent::CloseRequested => {
                let callback = self.with_graph(|graph| graph.on_close_requested.clone());
                if let Some(callback) = callback {
                    self.call_callback_sync(&callback, vec![app])?;
                }
                self.with_graph(|graph| graph.window_closed = true);
                Ok(10)
            }
            UiEvent::Paste(_) | UiEvent::FocusGained | UiEvent::FocusLost => Ok(0),
        }
    }

    fn dispatch_graph_redraw(&self, location: SourceLocation) -> Result<i64, VmError> {
        let pending = self.with_graph(|graph| graph.session.peek_redraw_pending(location))?;
        if !pending {
            return Ok(0);
        }
        let callback = self.with_graph(|graph| {
            graph
                .session
                .take_redraw_pending(location)
                .map(|_| graph.on_paint.clone())
        })?;
        if let Some(callback) = callback {
            self.call_callback_sync(&callback, vec![records::application()])?;
            self.with_graph(|graph| graph.session.present(location))?;
            Ok(5)
        } else {
            Ok(6)
        }
    }

    fn run_graph_application(&self, location: SourceLocation) -> Result<(), VmError> {
        let configured = self.with_graph(|graph| {
            if graph.run_active {
                return false;
            }
            graph.run_active = graph.on_paint.is_some();
            graph.run_active
        });
        if !configured {
            return Err(self.runtime_error(
                RUNTIME_INTRINSIC_STACK_STATE_ERROR,
                "Application.Run requires an OnPaint handler and no active run",
                "Configure OnPaint once before calling Application.Run.",
            ));
        }
        self.with_graph(|graph| graph.session.request_redraw_if_absent(location))?;
        let result = loop {
            self.dispatch_graph_redraw(location)?;
            self.process_graph_event(64, location)?;
            let stop = self.with_graph(|graph| {
                if graph.window_closed {
                    Some("WindowClosed")
                } else if graph.host_stop_requested && graph.quit_requested {
                    Some("HostAndUserStop")
                } else if graph.host_stop_requested {
                    Some("HostStop")
                } else if graph.quit_requested {
                    Some("UserQuit")
                } else {
                    None
                }
            });
            if let Some(reason) = stop {
                break records::exit_reason(reason);
            }
            let (idle, timeout) = self.with_graph(|graph| {
                (
                    graph.on_idle.clone(),
                    if graph.idle_interval_ms > 0 {
                        graph.idle_interval_ms
                    } else {
                        50
                    },
                )
            });
            let event = self
                .with_graph(|graph| graph.session.read_host_ui_event_timeout(timeout, location))?;
            if let Some(event) = event {
                self.with_graph(|graph| graph.host.ingest_ui_event(event));
            } else if let Some(callback) = idle {
                self.call_callback_sync(&callback, vec![records::application()])?;
            }
        };
        let on_exit = self.with_graph(|graph| {
            graph.last_exit_reason = Some(result.clone());
            graph.on_exit.clone()
        });
        if let Some(callback) = on_exit {
            self.call_callback_sync(&callback, vec![records::application(), result])?;
        }
        self.close_graph(location)
    }

    fn graph_host_arity_error(&self) -> VmError {
        self.runtime_error(
            RUNTIME_INTRINSIC_STACK_STATE_ERROR,
            "Graph host intrinsic is missing a verified argument",
            "Check the register intrinsic signature.",
        )
    }

    fn graph_host_type_error(&self, expected: &str, actual: &Value) -> VmError {
        self.runtime_error(
            RUNTIME_VM_OPERAND_TYPE_MISMATCH,
            format!("Expected {expected}, got {}", actual.type_name()),
            format!("Pass a {expected} value to this graph host operation."),
        )
    }
}

fn record_fields<'a>(
    value: Option<&'a Value>,
    expected: &str,
    worker: &RegisterWorker,
) -> Result<Vec<(&'a str, &'a Value)>, VmError> {
    match value {
        Some(Value::Record(record))
            if record.type_name == expected || record.type_name == "<record>" =>
        {
            Ok(record
                .fields
                .iter()
                .map(|(name, value)| (name.as_str(), value))
                .collect())
        }
        Some(Value::PositionalRecord(record)) => Ok(record
            .body()
            .layout
            .fields
            .iter()
            .map(String::as_str)
            .zip(&record.body().values)
            .collect()),
        actual => Err(worker.runtime_error(
            RUNTIME_VM_OPERAND_TYPE_MISMATCH,
            format!(
                "Expected {expected}, got {}",
                actual.map_or("missing", Value::type_name)
            ),
            format!("Pass a {expected} record."),
        )),
    }
}

fn required<'a>(
    fields: &'a [(&'a str, &'a Value)],
    name: &str,
    worker: &RegisterWorker,
) -> Result<&'a Value, VmError> {
    fields
        .iter()
        .find(|(field, _)| field.eq_ignore_ascii_case(name))
        .map(|(_, value)| *value)
        .ok_or_else(|| {
            worker.runtime_error(
                RUNTIME_VM_OPERAND_TYPE_MISMATCH,
                format!("ApplicationHandlers is missing field `{name}`"),
                "Construct the record with all declared fields.",
            )
        })
}

fn optional(
    fields: &[(&str, &Value)],
    name: &str,
    worker: &RegisterWorker,
) -> Result<Option<Value>, VmError> {
    match required(fields, name, worker)? {
        Value::OptionNone => Ok(None),
        Value::OptionSome(value) => Ok(Some((**value).clone())),
        actual => Err(worker.graph_host_type_error("option of function", actual)),
    }
}
