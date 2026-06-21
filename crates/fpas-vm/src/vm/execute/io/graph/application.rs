//! `Std.Graph` application lifecycle, hosted dispatch, and drawing intrinsics.
//!
//! **Documentation:** `docs/pascal/std/graph/session.md`, `docs/pascal/std/graph/app/README.md` (from the repository root).

use crate::vm::Worker;
use crate::vm::diagnostics::{TYPE_MISMATCH_CODE, VmError};
use crate::vm::runtime_error;
use fpas_bytecode::{GraphIntrinsic, Intrinsic, SourceLocation, Value};
use fpas_diagnostics::codes::RUNTIME_INTRINSIC_STACK_STATE_ERROR;

impl Worker {
    /// Executes application-level `Std.Graph` intrinsics.
    pub(super) fn try_exec_graph_application_intrinsic(
        &mut self,
        intrinsic: Intrinsic,
        line: SourceLocation,
    ) -> Result<bool, VmError> {
        if matches!(
            intrinsic,
            Intrinsic::Graph(
                GraphIntrinsic::ApplicationOpen
                    | GraphIntrinsic::ApplicationClose
                    | GraphIntrinsic::ApplicationSize
                    | GraphIntrinsic::ApplicationRequestRedraw
                    | GraphIntrinsic::ApplicationConfigure
                    | GraphIntrinsic::ApplicationRun
                    | GraphIntrinsic::ApplicationUploadFrame
                    | GraphIntrinsic::ApplicationClear
                    | GraphIntrinsic::ApplicationPutPixel
                    | GraphIntrinsic::ApplicationPresent
                    | GraphIntrinsic::ApplicationDrawLine
                    | GraphIntrinsic::ApplicationDrawRect
                    | GraphIntrinsic::ApplicationFillRect
                    | GraphIntrinsic::ApplicationDrawCircle
                    | GraphIntrinsic::ApplicationDrawText
            )
        ) {
            self.ensure_graph_main_task(line)?;
        }

        match intrinsic {
            Intrinsic::Graph(GraphIntrinsic::ApplicationOpen) => {
                let title = self.pop_graph_string(
                    "Pass a string title to `Std.Graph.Application.Open(Width, Height, Title)`.",
                    line,
                )?;
                let height = self.pop_int(line)?;
                let width = self.pop_int(line)?;
                {
                    let mut graph = self.shared.graph.lock().unwrap_or_else(|e| e.into_inner());
                    graph.session.open(width, height, &title, line)?;
                    let pending = std::mem::take(&mut graph.pending_test_events);
                    for event in pending {
                        graph.session.push_event(event, line)?;
                    }
                }
                self.push(Self::graph_application_record())?;
            }
            Intrinsic::Graph(GraphIntrinsic::ApplicationClose) => {
                self.pop_graph_application(line)?;
                if !self.request_graph_host_stop_for_active_run() {
                    self.close_graph_application_state(line)?;
                }
            }
            Intrinsic::Graph(GraphIntrinsic::ApplicationSize) => {
                self.pop_graph_application(line)?;
                let (width, height) = {
                    let mut graph = self.shared.graph.lock().unwrap_or_else(|e| e.into_inner());
                    graph.session.size(line)?
                };
                self.push(Self::graph_size_record(width, height))?;
            }
            Intrinsic::Graph(GraphIntrinsic::ApplicationRequestRedraw) => {
                self.pop_graph_application(line)?;
                self.with_graph(|graph| graph.session.request_redraw(line))?;
            }
            Intrinsic::Graph(GraphIntrinsic::ApplicationConfigure) => {
                let handlers = self.pop_graph_application_handlers(line)?;
                self.pop_graph_application(line)?;

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
                    "Set `OnMouse := Some(Handler)` or `None`; the handler must be `procedure (Application, Std.Graph.Event)`.",
                    line,
                )?;
                let on_wheel = self.optional_host_handler_field(
                    &handlers,
                    "OnWheel",
                    2,
                    "OnWheel",
                    "Set `OnWheel := Some(Handler)` or `None`; the handler must be `procedure (Application, Std.Graph.Event)`.",
                    line,
                )?;
                let on_resize = self.optional_host_handler_field(
                    &handlers,
                    "OnResize",
                    2,
                    "OnResize",
                    "Set `OnResize := Some(Handler)` or `None`; the handler must be `procedure (Application, Std.Graph.Size)`.",
                    line,
                )?;
                let on_close_requested = self.optional_host_handler_field(
                    &handlers,
                    "OnCloseRequested",
                    1,
                    "OnCloseRequested",
                    "Set `OnCloseRequested := Some(Handler)` or `None`; the handler must be `procedure (Application)`.",
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
                    "Set `OnExit := Some(Handler)` or `None`; the handler must be `procedure (Application, Std.Graph.ExitReason)`.",
                    line,
                )?;

                self.with_graph(|graph| {
                    graph.on_paint = Some(on_paint);
                    graph.on_key_pressed = on_key_pressed;
                    graph.on_mouse = on_mouse;
                    graph.on_wheel = on_wheel;
                    graph.on_resize = on_resize;
                    graph.on_close_requested = on_close_requested;
                    graph.idle_interval_ms = idle_interval_ms;
                    graph.on_idle = on_idle;
                    graph.on_exit = on_exit;
                });
            }
            Intrinsic::Graph(GraphIntrinsic::ApplicationRun) => {
                self.graph_application_run(line)?;
                self.push(Value::Unit)?;
            }
            Intrinsic::Graph(GraphIntrinsic::ApplicationUploadFrame) => {
                let pixels = self.pop_graph_pixels(line)?;
                let height = self.pop_int(line)?;
                let width = self.pop_int(line)?;
                self.pop_graph_application(line)?;
                let mut graph = self.shared.graph.lock().unwrap_or_else(|e| e.into_inner());
                graph.session.upload_frame(width, height, &pixels, line)?;
            }
            Intrinsic::Graph(GraphIntrinsic::ApplicationClear) => {
                let color = self.pop_int(line)?;
                self.pop_graph_application(line)?;
                let mut graph = self.shared.graph.lock().unwrap_or_else(|e| e.into_inner());
                graph.session.clear(color, line)?;
            }
            Intrinsic::Graph(GraphIntrinsic::ApplicationPutPixel) => {
                let color = self.pop_int(line)?;
                let y = self.pop_int(line)?;
                let x = self.pop_int(line)?;
                self.pop_graph_application(line)?;
                let mut graph = self.shared.graph.lock().unwrap_or_else(|e| e.into_inner());
                graph.session.put_pixel(x, y, color, line)?;
            }
            Intrinsic::Graph(GraphIntrinsic::ApplicationPresent) => {
                self.pop_graph_application(line)?;
                let mut graph = self.shared.graph.lock().unwrap_or_else(|e| e.into_inner());
                graph.session.present(line)?;
            }
            Intrinsic::Graph(GraphIntrinsic::ApplicationDrawLine) => {
                let color = self.pop_int(line)?;
                let y2 = self.pop_int(line)?;
                let x2 = self.pop_int(line)?;
                let y1 = self.pop_int(line)?;
                let x1 = self.pop_int(line)?;
                self.pop_graph_application(line)?;
                let mut graph = self.shared.graph.lock().unwrap_or_else(|e| e.into_inner());
                graph.session.draw_line(x1, y1, x2, y2, color, line)?;
            }
            Intrinsic::Graph(GraphIntrinsic::ApplicationDrawRect) => {
                let color = self.pop_int(line)?;
                let height = self.pop_int(line)?;
                let width = self.pop_int(line)?;
                let y = self.pop_int(line)?;
                let x = self.pop_int(line)?;
                self.pop_graph_application(line)?;
                let mut graph = self.shared.graph.lock().unwrap_or_else(|e| e.into_inner());
                graph.session.draw_rect(x, y, width, height, color, line)?;
            }
            Intrinsic::Graph(GraphIntrinsic::ApplicationFillRect) => {
                let color = self.pop_int(line)?;
                let height = self.pop_int(line)?;
                let width = self.pop_int(line)?;
                let y = self.pop_int(line)?;
                let x = self.pop_int(line)?;
                self.pop_graph_application(line)?;
                let mut graph = self.shared.graph.lock().unwrap_or_else(|e| e.into_inner());
                graph.session.fill_rect(x, y, width, height, color, line)?;
            }
            Intrinsic::Graph(GraphIntrinsic::ApplicationDrawCircle) => {
                let color = self.pop_int(line)?;
                let radius = self.pop_int(line)?;
                let center_y = self.pop_int(line)?;
                let center_x = self.pop_int(line)?;
                self.pop_graph_application(line)?;
                let mut graph = self.shared.graph.lock().unwrap_or_else(|e| e.into_inner());
                graph
                    .session
                    .draw_circle(center_x, center_y, radius, color, line)?;
            }
            Intrinsic::Graph(GraphIntrinsic::ApplicationDrawText) => {
                let color = self.pop_int(line)?;
                let text = self.pop_graph_text(line)?;
                let y = self.pop_int(line)?;
                let x = self.pop_int(line)?;
                self.pop_graph_application(line)?;
                let mut graph = self.shared.graph.lock().unwrap_or_else(|e| e.into_inner());
                graph.session.draw_text(x, y, &text, color, line)?;
            }
            _ => return Ok(false),
        }

        Ok(true)
    }

    fn pop_graph_string(
        &mut self,
        help: &'static str,
        line: SourceLocation,
    ) -> Result<String, VmError> {
        match self.pop(line)? {
            Value::Str(title) => Ok(title),
            other => Err(runtime_error(
                TYPE_MISMATCH_CODE,
                format!("Expected string, got {}", other.type_name()),
                help,
                line,
            )),
        }
    }

    fn pop_graph_text(&mut self, line: SourceLocation) -> Result<String, VmError> {
        match self.pop(line)? {
            Value::Str(text) => Ok(text),
            other => Err(runtime_error(
                TYPE_MISMATCH_CODE,
                format!("Expected string, got {}", other.type_name()),
                "Pass a string as `Text` to `Std.Graph.Application.DrawText(App, X, Y, Text, Color)`.",
                line,
            )),
        }
    }

    fn pop_graph_pixels(&mut self, line: SourceLocation) -> Result<Vec<i64>, VmError> {
        let values = match self.pop(line)? {
            Value::Array(values) => values,
            other => {
                return Err(runtime_error(
                    TYPE_MISMATCH_CODE,
                    format!("Expected array, got {}", other.type_name()),
                    "Pass an `array of integer` as the `Pixels` argument to `Std.Graph.Application.UploadFrame(App, Width, Height, Pixels)`.",
                    line,
                ));
            }
        };

        values
            .into_iter()
            .map(|value| match value {
                Value::Integer(pixel) => Ok(pixel),
                other => Err(runtime_error(
                    TYPE_MISMATCH_CODE,
                    format!(
                        "Std.Graph.Application.UploadFrame(App, Width, Height, Pixels) expects `Pixels` to contain only integer values, but found {}.",
                        other.type_name()
                    ),
                    "Build `Pixels` as `array of integer` with packed `$00RRGGBB` values.",
                    line,
                )),
            })
            .collect()
    }

    fn ensure_graph_main_task(&self, line: SourceLocation) -> Result<(), VmError> {
        if self.current_task_id == 0 {
            Ok(())
        } else {
            Err(runtime_error(
                RUNTIME_INTRINSIC_STACK_STATE_ERROR,
                "Std.Graph.Application.* must run on the main task",
                "Call `Std.Graph.Application.*` from the main program, not from a `go` task.",
                line,
            ))
        }
    }
}
