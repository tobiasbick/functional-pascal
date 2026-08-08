//! FPAS records used at the register `Std.Graph` host boundary.

use fpas_bytecode::Value;
use fpas_std::{
    ConsoleKeyEvent, GraphEvent, GraphEventKind, mouse_action_index, mouse_button_index,
};

use crate::vm::execute::io::console_records::key_event_record;

pub(super) fn application() -> Value {
    Value::record("Std.Graph.Application".into(), vec![])
}

pub(super) fn size(width: i64, height: i64) -> Value {
    Value::record(
        "Std.Graph.Size".into(),
        vec![
            ("width".into(), Value::Integer(width)),
            ("height".into(), Value::Integer(height)),
        ],
    )
}

pub(super) fn exit_reason(variant: &str) -> Value {
    Value::enum_value("Std.Graph.ExitReason".into(), variant.into(), vec![])
}

pub(super) fn event(event: GraphEvent) -> Value {
    match event {
        GraphEvent::CloseRequested => event_with(GraphEventKind::CloseRequested, (0, 0), &[]),
        GraphEvent::Resize { width, height } => {
            event_with(GraphEventKind::Resize, (width, height), &[])
        }
        GraphEvent::Key(key) => event_with(
            GraphEventKind::Key,
            (0, 0),
            &[
                ("key".into(), key_event_record(key.clone())),
                ("shift".into(), Value::Boolean(key.shift)),
                ("ctrl".into(), Value::Boolean(key.ctrl)),
                ("alt".into(), Value::Boolean(key.alt)),
                ("meta".into(), Value::Boolean(key.meta)),
            ],
        ),
        GraphEvent::Mouse {
            action,
            button,
            x,
            y,
            shift,
            ctrl,
            alt,
            meta,
        } => event_with(
            GraphEventKind::Mouse,
            (0, 0),
            &[
                ("mouse_action".into(), Value::Integer(action as i64)),
                ("mouse_button".into(), Value::Integer(button as i64)),
                ("mouse_x".into(), Value::Integer(x)),
                ("mouse_y".into(), Value::Integer(y)),
                ("shift".into(), Value::Boolean(shift)),
                ("ctrl".into(), Value::Boolean(ctrl)),
                ("alt".into(), Value::Boolean(alt)),
                ("meta".into(), Value::Boolean(meta)),
            ],
        ),
        GraphEvent::Wheel {
            delta_x,
            delta_y,
            x,
            y,
            shift,
            ctrl,
            alt,
            meta,
        } => event_with(
            GraphEventKind::Wheel,
            (0, 0),
            &[
                ("mouse_x".into(), Value::Integer(x)),
                ("mouse_y".into(), Value::Integer(y)),
                ("wheel_x".into(), Value::Integer(delta_x)),
                ("wheel_y".into(), Value::Integer(delta_y)),
                ("shift".into(), Value::Boolean(shift)),
                ("ctrl".into(), Value::Boolean(ctrl)),
                ("alt".into(), Value::Boolean(alt)),
                ("meta".into(), Value::Boolean(meta)),
            ],
        ),
    }
}

fn event_with(
    kind: GraphEventKind,
    dimensions: (i64, i64),
    overrides: &[(String, Value)],
) -> Value {
    let kind = match kind {
        GraphEventKind::CloseRequested => 0,
        GraphEventKind::Resize => 1,
        GraphEventKind::Key => 2,
        GraphEventKind::Mouse => 3,
        GraphEventKind::Wheel => 4,
    };
    let mut fields = vec![
        ("kind".into(), Value::Integer(kind)),
        ("size".into(), size(dimensions.0, dimensions.1)),
        (
            "key".into(),
            key_event_record(ConsoleKeyEvent::new(
                fpas_std::key_event::key_kind_index("Unknown"),
                '\0',
                false,
                false,
                false,
                false,
            )),
        ),
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
    ];
    for (name, value) in overrides {
        if let Some(field) = fields.iter_mut().find(|(field, _)| field == name) {
            field.1 = value.clone();
        }
    }
    Value::record("Std.Graph.Event".into(), fields)
}
