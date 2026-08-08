//! FPAS records used at the register `Std.Graph` host boundary.

use fpas_bytecode::{SourceLocation, Value};
use fpas_std::{
    ConsoleKeyEvent, GraphEvent, GraphEventKind, mouse_action_index, mouse_button_index,
};

use crate::vm::hosted::console_records::key_event_record;
use crate::vm::{VmError, worker::Worker};

pub(super) fn application(worker: &Worker, location: SourceLocation) -> Result<Value, VmError> {
    worker.record_value("Std.Graph.Application", vec![], location)
}

pub(super) fn size(
    worker: &Worker,
    width: i64,
    height: i64,
    location: SourceLocation,
) -> Result<Value, VmError> {
    worker.record_value(
        "Std.Graph.Size",
        vec![Value::Integer(width), Value::Integer(height)],
        location,
    )
}

pub(super) fn exit_reason(worker: &Worker, variant: &str) -> Result<Value, VmError> {
    let discriminant = fpas_std::GRAPH_EXIT_REASON_VARIANTS
        .iter()
        .position(|candidate| candidate.eq_ignore_ascii_case(variant))
        .ok_or_else(|| {
            worker.runtime_error(
                fpas_diagnostics::codes::RUNTIME_INTRINSIC_STACK_STATE_ERROR,
                format!("Unknown Std.Graph.ExitReason variant `{variant}`"),
                "This indicates a hosted-runtime/compiler mismatch.",
            )
        })?;
    let discriminant = i64::try_from(discriminant).map_err(|_| {
        worker.runtime_error(
            fpas_diagnostics::codes::RUNTIME_INTRINSIC_STACK_STATE_ERROR,
            "Std.Graph.ExitReason discriminant exceeds the runtime integer range",
            "This indicates a hosted-runtime/compiler mismatch.",
        )
    })?;
    Ok(Value::Integer(discriminant))
}

pub(super) fn event(
    worker: &Worker,
    event: GraphEvent,
    location: SourceLocation,
) -> Result<Value, VmError> {
    match event {
        GraphEvent::CloseRequested => event_with(
            worker,
            GraphEventKind::CloseRequested,
            (0, 0),
            &[],
            location,
        ),
        GraphEvent::Resize { width, height } => event_with(
            worker,
            GraphEventKind::Resize,
            (width, height),
            &[],
            location,
        ),
        GraphEvent::Key(key) => event_with(
            worker,
            GraphEventKind::Key,
            (0, 0),
            &[
                (
                    "key".into(),
                    key_event_record(worker, key.clone(), location)?,
                ),
                ("shift".into(), Value::Boolean(key.shift)),
                ("ctrl".into(), Value::Boolean(key.ctrl)),
                ("alt".into(), Value::Boolean(key.alt)),
                ("meta".into(), Value::Boolean(key.meta)),
            ],
            location,
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
            worker,
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
            location,
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
            worker,
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
            location,
        ),
    }
}

fn event_with(
    worker: &Worker,
    kind: GraphEventKind,
    dimensions: (i64, i64),
    overrides: &[(String, Value)],
    location: SourceLocation,
) -> Result<Value, VmError> {
    let kind = match kind {
        GraphEventKind::CloseRequested => 0,
        GraphEventKind::Resize => 1,
        GraphEventKind::Key => 2,
        GraphEventKind::Mouse => 3,
        GraphEventKind::Wheel => 4,
    };
    let mut fields: Vec<(String, Value)> = vec![
        ("kind".into(), Value::Integer(kind)),
        (
            "size".into(),
            size(worker, dimensions.0, dimensions.1, location)?,
        ),
        (
            "key".into(),
            key_event_record(
                worker,
                ConsoleKeyEvent::new(
                    fpas_std::key_event::key_kind_index("Unknown"),
                    '\0',
                    false,
                    false,
                    false,
                    false,
                ),
                location,
            )?,
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
    worker.record_value(
        "Std.Graph.Event",
        fields.into_iter().map(|(_, value)| value).collect(),
        location,
    )
}
