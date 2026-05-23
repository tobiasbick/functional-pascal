//! `Std.Graph` application lifecycle, event, and upload intrinsics.
//!
//! **Documentation:** `docs/future/std.graph/02-pascal-surface.md`, `docs/future/std.graph/04-implementation-plan.md` (from the repository root).

use crate::vm::Worker;
use crate::vm::diagnostics::{TYPE_MISMATCH_CODE, VmError};
use crate::vm::runtime_error;
use fpas_bytecode::{GraphIntrinsic, Intrinsic, SourceLocation, Value};
use fpas_diagnostics::codes::RUNTIME_INTRINSIC_STACK_STATE_ERROR;

impl Worker {
    /// Executes application-level `Std.Graph` intrinsics through the shared graph session.
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
                    | GraphIntrinsic::ApplicationPollEvent
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
                let mut graph = self.shared.graph.lock().unwrap_or_else(|e| e.into_inner());
                graph.session.close(line)?;
            }
            Intrinsic::Graph(GraphIntrinsic::ApplicationSize) => {
                self.pop_graph_application(line)?;
                let (width, height) = {
                    let graph = self.shared.graph.lock().unwrap_or_else(|e| e.into_inner());
                    graph.session.size(line)?
                };
                self.push(Self::graph_size_record(width, height))?;
            }
            Intrinsic::Graph(GraphIntrinsic::ApplicationPollEvent) => {
                self.pop_graph_application(line)?;
                let event = {
                    let mut graph = self.shared.graph.lock().unwrap_or_else(|e| e.into_inner());
                    graph.session.poll_event(line)?
                };
                match event {
                    Some(event) => {
                        self.push(Value::OptionSome(Box::new(Self::graph_event_record(event))))?
                    }
                    None => self.push(Value::OptionNone)?,
                }
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
            Value::Char(ch) => Ok(ch.to_string()),
            other => Err(runtime_error(
                TYPE_MISMATCH_CODE,
                format!("Expected string or char, got {}", other.type_name()),
                "Pass a string or single character as `Text` to `Std.Graph.Application.DrawText(App, X, Y, Text, Color)`.",
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
