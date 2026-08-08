//! Borrowed `Std.Graph` register intrinsics over the shared platform host.

mod host;
mod records;

use fpas_bytecode::{GraphIntrinsic, Intrinsic, SourceLocation, Value};
use fpas_diagnostics::codes::{
    RUNTIME_CONSOLE_STATE_ERROR, RUNTIME_INTRINSIC_STACK_STATE_ERROR,
    RUNTIME_VM_OPERAND_TYPE_MISMATCH,
};
use fpas_std::{GraphEvent, HeadlessGraphTestModeGuard};

use crate::vm::VmError;
use crate::vm::hosted::console_records::console_key_event_from_value;
use crate::vm::worker::Worker;

impl Worker {
    pub(super) fn execute_graph_intrinsic(
        &self,
        intrinsic: Intrinsic,
        arguments: &[Value],
        location: SourceLocation,
    ) -> Result<Option<Option<Value>>, VmError> {
        let Intrinsic::Graph(operation) = intrinsic else {
            return Ok(None);
        };
        if let Some(result) = self.execute_graph_host(operation, arguments, location)? {
            return Ok(Some(result));
        }
        let result = match operation {
            GraphIntrinsic::ApplicationOpen => {
                let width = integer(arguments, 0, 3, self)?;
                let height = integer(arguments, 1, 3, self)?;
                let title = string(arguments, 2, 3, self)?;
                self.with_graph(|graph| {
                    graph.session.open(width, height, title, location)?;
                    for event in std::mem::take(&mut graph.pending_test_events) {
                        graph.session.push_event(event, location)?;
                    }
                    Ok(())
                })?;
                Some(records::application(self, location)?)
            }
            GraphIntrinsic::OpenForTest => {
                let width = positive_dimension(arguments, 0, self)?;
                let height = positive_dimension(arguments, 1, self)?;
                let guard = HeadlessGraphTestModeGuard::push();
                let opened = self.with_graph(|graph| {
                    if graph.session.is_open() {
                        return Err(self.runtime_error(
                            RUNTIME_INTRINSIC_STACK_STATE_ERROR,
                            "Application.OpenForTest cannot open a second graph session",
                            "Close the active application before opening another one.",
                        ));
                    }
                    graph.session.open(width, height, "", location)?;
                    graph.headless_test_open = true;
                    for event in std::mem::take(&mut graph.pending_test_events) {
                        graph.session.push_event(event, location)?;
                    }
                    Ok(())
                });
                opened?;
                guard.release();
                Some(records::application(self, location)?)
            }
            GraphIntrinsic::ApplicationClose => {
                application(arguments.first(), self)?;
                let running = self.with_graph(|graph| {
                    if graph.run_active {
                        graph.host_stop_requested = true;
                        true
                    } else {
                        false
                    }
                });
                if !running {
                    self.close_graph(location)?;
                }
                None
            }
            GraphIntrinsic::ApplicationSize => {
                application(arguments.first(), self)?;
                let (width, height) = self.with_graph(|graph| graph.session.size(location))?;
                Some(records::size(self, width, height, location)?)
            }
            GraphIntrinsic::ApplicationRequestRedraw => {
                application(arguments.first(), self)?;
                self.with_graph(|graph| graph.session.request_redraw(location))?;
                None
            }
            GraphIntrinsic::ApplicationUploadFrame => {
                application(arguments.first(), self)?;
                let width = integer(arguments, 1, 4, self)?;
                let height = integer(arguments, 2, 4, self)?;
                let pixels = pixels(arguments.get(3), self)?;
                self.with_graph(|graph| {
                    graph.session.upload_frame(width, height, &pixels, location)
                })?;
                None
            }
            GraphIntrinsic::ApplicationClear => {
                application(arguments.first(), self)?;
                let color = integer(arguments, 1, 2, self)?;
                self.with_graph(|graph| graph.session.clear(color, location))?;
                None
            }
            GraphIntrinsic::ApplicationPutPixel => {
                application(arguments.first(), self)?;
                self.with_graph(|graph| {
                    graph.session.put_pixel(
                        integer(arguments, 1, 4, self)?,
                        integer(arguments, 2, 4, self)?,
                        integer(arguments, 3, 4, self)?,
                        location,
                    )
                })?;
                None
            }
            GraphIntrinsic::ApplicationPresent => {
                application(arguments.first(), self)?;
                self.with_graph(|graph| graph.session.present(location))?;
                None
            }
            GraphIntrinsic::ApplicationDrawLine => {
                application(arguments.first(), self)?;
                self.with_graph(|graph| {
                    graph.session.draw_line(
                        integer(arguments, 1, 6, self)?,
                        integer(arguments, 2, 6, self)?,
                        integer(arguments, 3, 6, self)?,
                        integer(arguments, 4, 6, self)?,
                        integer(arguments, 5, 6, self)?,
                        location,
                    )
                })?;
                None
            }
            GraphIntrinsic::ApplicationDrawRect | GraphIntrinsic::ApplicationFillRect => {
                application(arguments.first(), self)?;
                let x = integer(arguments, 1, 6, self)?;
                let y = integer(arguments, 2, 6, self)?;
                let width = integer(arguments, 3, 6, self)?;
                let height = integer(arguments, 4, 6, self)?;
                let color = integer(arguments, 5, 6, self)?;
                self.with_graph(|graph| {
                    if operation == GraphIntrinsic::ApplicationDrawRect {
                        graph
                            .session
                            .draw_rect(x, y, width, height, color, location)
                    } else {
                        graph
                            .session
                            .fill_rect(x, y, width, height, color, location)
                    }
                })?;
                None
            }
            GraphIntrinsic::ApplicationDrawCircle => {
                application(arguments.first(), self)?;
                self.with_graph(|graph| {
                    graph.session.draw_circle(
                        integer(arguments, 1, 5, self)?,
                        integer(arguments, 2, 5, self)?,
                        integer(arguments, 3, 5, self)?,
                        integer(arguments, 4, 5, self)?,
                        location,
                    )
                })?;
                None
            }
            GraphIntrinsic::ApplicationDrawText => {
                application(arguments.first(), self)?;
                self.with_graph(|graph| {
                    graph.session.draw_text(
                        integer(arguments, 1, 5, self)?,
                        integer(arguments, 2, 5, self)?,
                        string(arguments, 3, 5, self)?,
                        integer(arguments, 4, 5, self)?,
                        location,
                    )
                })?;
                None
            }
            GraphIntrinsic::TestSendKey => {
                application(arguments.first(), self)?;
                let key = console_key_event_from_value(value(arguments, 1, 2, self)?, location)?;
                self.with_graph(|graph| {
                    if graph.session.is_open() {
                        graph.session.push_event(GraphEvent::Key(key), location)
                    } else {
                        graph.pending_test_events.push(GraphEvent::Key(key));
                        Ok(())
                    }
                })?;
                None
            }
            _ => return Ok(None),
        };
        Ok(Some(result))
    }

    pub(super) fn with_graph<R>(
        &self,
        operation: impl FnOnce(&mut crate::vm::GraphState) -> R,
    ) -> R {
        operation(
            &mut self
                .hosted
                .graph
                .lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner()),
        )
    }

    fn close_graph(&self, location: SourceLocation) -> Result<(), VmError> {
        self.with_graph(|graph| {
            let result = graph.session.close(location);
            graph.host = fpas_std::UiHost::for_graph();
            graph.on_key_pressed = None;
            graph.on_mouse = None;
            graph.on_wheel = None;
            graph.on_resize = None;
            graph.on_close_requested = None;
            graph.on_paint = None;
            graph.on_idle = None;
            graph.on_exit = None;
            graph.idle_interval_ms = 0;
            graph.quit_requested = false;
            graph.window_closed = false;
            graph.host_stop_requested = false;
            graph.run_active = false;
            graph.pending_test_events.clear();
            if graph.headless_test_open {
                fpas_std::pop_headless_graph_test_mode();
                graph.headless_test_open = false;
            }
            result
        })?;
        Ok(())
    }
}

fn value<'a>(
    arguments: &'a [Value],
    index: usize,
    count: usize,
    worker: &Worker,
) -> Result<&'a Value, VmError> {
    if arguments.len() != count {
        return Err(worker.runtime_error(
            RUNTIME_INTRINSIC_STACK_STATE_ERROR,
            format!(
                "Graph intrinsic expected {count} arguments, got {}",
                arguments.len()
            ),
            "Check the verified register intrinsic signature.",
        ));
    }
    arguments.get(index).ok_or_else(|| {
        worker.runtime_error(
            RUNTIME_INTRINSIC_STACK_STATE_ERROR,
            "Graph intrinsic argument is missing",
            "Check the verified register intrinsic signature.",
        )
    })
}

pub(super) fn integer(
    arguments: &[Value],
    index: usize,
    count: usize,
    worker: &Worker,
) -> Result<i64, VmError> {
    match value(arguments, index, count, worker)? {
        Value::Integer(value) => Ok(*value),
        actual => Err(worker.runtime_error(
            RUNTIME_VM_OPERAND_TYPE_MISMATCH,
            format!(
                "Graph intrinsic expected integer, got {}",
                actual.type_name()
            ),
            "Pass an integer value to this Std.Graph call.",
        )),
    }
}

pub(super) fn string<'a>(
    arguments: &'a [Value],
    index: usize,
    count: usize,
    worker: &Worker,
) -> Result<&'a str, VmError> {
    match value(arguments, index, count, worker)? {
        Value::Str(value) => Ok(value),
        actual => Err(worker.runtime_error(
            RUNTIME_VM_OPERAND_TYPE_MISMATCH,
            format!(
                "Graph intrinsic expected string, got {}",
                actual.type_name()
            ),
            "Pass a string value to this Std.Graph call.",
        )),
    }
}

pub(super) fn application(value: Option<&Value>, worker: &Worker) -> Result<(), VmError> {
    match value {
        Some(Value::Record(record))
            if record
                .body()
                .layout
                .type_name
                .eq_ignore_ascii_case("Std.Graph.Application") =>
        {
            Ok(())
        }
        actual => Err(worker.runtime_error(
            RUNTIME_VM_OPERAND_TYPE_MISMATCH,
            format!(
                "Expected Std.Graph.Application, got {}",
                actual.map_or("missing", Value::type_name)
            ),
            "Pass the Application returned by Open or OpenForTest.",
        )),
    }
}

fn positive_dimension(arguments: &[Value], index: usize, worker: &Worker) -> Result<i64, VmError> {
    let dimension = integer(arguments, index, 2, worker)?;
    if dimension > 0 {
        Ok(dimension)
    } else {
        Err(worker.runtime_error(
            RUNTIME_CONSOLE_STATE_ERROR,
            format!("Application.OpenForTest requires a positive dimension, got {dimension}"),
            "Pass positive pixel dimensions.",
        ))
    }
}

fn pixels(value: Option<&Value>, worker: &Worker) -> Result<Vec<i64>, VmError> {
    let Some(Value::Array(values)) = value else {
        return Err(worker.runtime_error(
            RUNTIME_VM_OPERAND_TYPE_MISMATCH,
            "UploadFrame expects an array of integer pixels",
            "Pass packed $00RRGGBB integer values.",
        ));
    };
    values
        .iter()
        .map(|value| match value {
            Value::Integer(value) => Ok(*value),
            actual => Err(worker.runtime_error(
                RUNTIME_VM_OPERAND_TYPE_MISMATCH,
                format!(
                    "UploadFrame pixel must be integer, got {}",
                    actual.type_name()
                ),
                "Pass packed $00RRGGBB integer values.",
            )),
        })
        .collect()
}
