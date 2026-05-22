//! `Std.Graph` record constructors and handle validation.
//!
//! **Documentation:** `docs/future/std.graph/02-pascal-surface.md`, `docs/future/std.graph/04-implementation-plan.md` (from the repository root).

use crate::vm::Worker;
use crate::vm::diagnostics::{TYPE_MISMATCH_CODE, VmError};
use crate::vm::runtime_error;
use fpas_bytecode::{SourceLocation, Value};
use fpas_std::{ConsoleKeyEvent, GraphEvent, GraphEventKind};

const GRAPH_APPLICATION_TYPE: &str = "Std.Graph.Application";
const GRAPH_EVENT_TYPE: &str = "Std.Graph.Event";
const GRAPH_SIZE_TYPE: &str = "Std.Graph.Size";

impl Worker {
    /// Constructs an empty `Std.Graph.Application` record.
    pub(in crate::vm::execute::io) fn graph_application_record() -> Value {
        Value::Record {
            type_name: GRAPH_APPLICATION_TYPE.into(),
            fields: vec![],
        }
    }

    /// Constructs a `Std.Graph.Size` record with `width` and `height` fields.
    pub(in crate::vm::execute::io) fn graph_size_record(width: i64, height: i64) -> Value {
        Value::Record {
            type_name: GRAPH_SIZE_TYPE.into(),
            fields: vec![
                ("width".into(), Value::Integer(width)),
                ("height".into(), Value::Integer(height)),
            ],
        }
    }

    /// Converts a normalized `GraphEvent` into a `Std.Graph.Event` record.
    pub(in crate::vm::execute::io) fn graph_event_record(event: GraphEvent) -> Value {
        match event {
            GraphEvent::CloseRequested => Value::Record {
                type_name: GRAPH_EVENT_TYPE.into(),
                fields: vec![
                    (
                        "kind".into(),
                        Self::graph_event_kind_value(GraphEventKind::CloseRequested),
                    ),
                    ("size".into(), Self::graph_size_record(0, 0)),
                    ("key".into(), Self::graph_unknown_key_event()),
                ],
            },
            GraphEvent::Resize { width, height } => Value::Record {
                type_name: GRAPH_EVENT_TYPE.into(),
                fields: vec![
                    (
                        "kind".into(),
                        Self::graph_event_kind_value(GraphEventKind::Resize),
                    ),
                    ("size".into(), Self::graph_size_record(width, height)),
                    ("key".into(), Self::graph_unknown_key_event()),
                ],
            },
            GraphEvent::Key(key) => Value::Record {
                type_name: GRAPH_EVENT_TYPE.into(),
                fields: vec![
                    (
                        "kind".into(),
                        Self::graph_event_kind_value(GraphEventKind::Key),
                    ),
                    ("size".into(), Self::graph_size_record(0, 0)),
                    ("key".into(), Self::key_event_record(key)),
                ],
            },
        }
    }

    /// Pops a `Std.Graph.Application` record from the stack.
    pub(in crate::vm::execute::io) fn pop_graph_application(
        &mut self,
        line: SourceLocation,
    ) -> Result<(), VmError> {
        match self.pop(line)? {
            Value::Record { type_name, .. } if type_name == GRAPH_APPLICATION_TYPE => Ok(()),
            other => Err(runtime_error(
                TYPE_MISMATCH_CODE,
                format!(
                    "Expected {GRAPH_APPLICATION_TYPE}, got {}",
                    other.type_name()
                ),
                "Pass the value returned by `Std.Graph.Application.Open()`.",
                line,
            )),
        }
    }

    fn graph_event_kind_value(kind: GraphEventKind) -> Value {
        let index = match kind {
            GraphEventKind::CloseRequested => 0,
            GraphEventKind::Resize => 1,
            GraphEventKind::Key => 2,
        };
        Value::Integer(index)
    }

    fn graph_unknown_key_event() -> Value {
        Self::key_event_record(ConsoleKeyEvent::new(
            fpas_std::key_event::key_kind_index("Unknown"),
            '\0',
            false,
            false,
            false,
            false,
        ))
    }
}
