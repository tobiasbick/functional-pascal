//! `Std.Graph` record constructors and handle validation.
//!
//! **Documentation:** `docs/future/std.graph/02-pascal-surface.md`, `docs/future/std.graph/04-implementation-plan.md` (from the repository root).

use crate::vm::Worker;
use crate::vm::diagnostics::{TYPE_MISMATCH_CODE, VmError};
use crate::vm::runtime_error;
use fpas_bytecode::{SourceLocation, Value};
use fpas_std::{
    ConsoleKeyEvent, GraphEvent, GraphEventKind, mouse_action_index, mouse_button_index,
};

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
                    (
                        "mouse_action".into(),
                        Value::Integer(mouse_action_index("Unknown") as i64),
                    ),
                    (
                        "mouse_button".into(),
                        Value::Integer(mouse_button_index("None") as i64),
                    ),
                    ("mouse_x".into(), Value::Integer(0)),
                    ("mouse_y".into(), Value::Integer(0)),
                    ("wheel_x".into(), Value::Integer(0)),
                    ("wheel_y".into(), Value::Integer(0)),
                    ("shift".into(), Value::Boolean(false)),
                    ("ctrl".into(), Value::Boolean(false)),
                    ("alt".into(), Value::Boolean(false)),
                    ("meta".into(), Value::Boolean(false)),
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
                    (
                        "mouse_action".into(),
                        Value::Integer(mouse_action_index("Unknown") as i64),
                    ),
                    (
                        "mouse_button".into(),
                        Value::Integer(mouse_button_index("None") as i64),
                    ),
                    ("mouse_x".into(), Value::Integer(0)),
                    ("mouse_y".into(), Value::Integer(0)),
                    ("wheel_x".into(), Value::Integer(0)),
                    ("wheel_y".into(), Value::Integer(0)),
                    ("shift".into(), Value::Boolean(false)),
                    ("ctrl".into(), Value::Boolean(false)),
                    ("alt".into(), Value::Boolean(false)),
                    ("meta".into(), Value::Boolean(false)),
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
                    ("key".into(), Self::key_event_record(key.clone())),
                    (
                        "mouse_action".into(),
                        Value::Integer(mouse_action_index("Unknown") as i64),
                    ),
                    (
                        "mouse_button".into(),
                        Value::Integer(mouse_button_index("None") as i64),
                    ),
                    ("mouse_x".into(), Value::Integer(0)),
                    ("mouse_y".into(), Value::Integer(0)),
                    ("wheel_x".into(), Value::Integer(0)),
                    ("wheel_y".into(), Value::Integer(0)),
                    ("shift".into(), Value::Boolean(key.shift)),
                    ("ctrl".into(), Value::Boolean(key.ctrl)),
                    ("alt".into(), Value::Boolean(key.alt)),
                    ("meta".into(), Value::Boolean(key.meta)),
                ],
            },
            GraphEvent::Mouse {
                action,
                button,
                x,
                y,
                shift,
                ctrl,
                alt,
                meta,
            } => Value::Record {
                type_name: GRAPH_EVENT_TYPE.into(),
                fields: vec![
                    (
                        "kind".into(),
                        Self::graph_event_kind_value(GraphEventKind::Mouse),
                    ),
                    ("size".into(), Self::graph_size_record(0, 0)),
                    ("key".into(), Self::graph_unknown_key_event()),
                    ("mouse_action".into(), Value::Integer(action as i64)),
                    ("mouse_button".into(), Value::Integer(button as i64)),
                    ("mouse_x".into(), Value::Integer(x)),
                    ("mouse_y".into(), Value::Integer(y)),
                    ("wheel_x".into(), Value::Integer(0)),
                    ("wheel_y".into(), Value::Integer(0)),
                    ("shift".into(), Value::Boolean(shift)),
                    ("ctrl".into(), Value::Boolean(ctrl)),
                    ("alt".into(), Value::Boolean(alt)),
                    ("meta".into(), Value::Boolean(meta)),
                ],
            },
            GraphEvent::Wheel {
                delta_x,
                delta_y,
                x,
                y,
                shift,
                ctrl,
                alt,
                meta,
            } => Value::Record {
                type_name: GRAPH_EVENT_TYPE.into(),
                fields: vec![
                    (
                        "kind".into(),
                        Self::graph_event_kind_value(GraphEventKind::Wheel),
                    ),
                    ("size".into(), Self::graph_size_record(0, 0)),
                    ("key".into(), Self::graph_unknown_key_event()),
                    (
                        "mouse_action".into(),
                        Value::Integer(mouse_action_index("Unknown") as i64),
                    ),
                    (
                        "mouse_button".into(),
                        Value::Integer(mouse_button_index("None") as i64),
                    ),
                    ("mouse_x".into(), Value::Integer(x)),
                    ("mouse_y".into(), Value::Integer(y)),
                    ("wheel_x".into(), Value::Integer(delta_x)),
                    ("wheel_y".into(), Value::Integer(delta_y)),
                    ("shift".into(), Value::Boolean(shift)),
                    ("ctrl".into(), Value::Boolean(ctrl)),
                    ("alt".into(), Value::Boolean(alt)),
                    ("meta".into(), Value::Boolean(meta)),
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
            GraphEventKind::Mouse => 3,
            GraphEventKind::Wheel => 4,
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
