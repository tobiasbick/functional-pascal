//! Conversion of typed VM snapshots using the shared diagnostic value serializer.

use super::diagnostic;
use crate::snapshot_value::Value as CapturedValue;
use fpas_vm::{DebugSnapshot, DebugTaskSnapshot, FailureBinding, FailureFrame};
use serde_json::{Value, json};

#[expect(
    clippy::unwrap_used,
    reason = "The caller supplies the empty_report envelope with its diagnostics array."
)]
pub(super) fn append(report: &mut Value, state: DebugSnapshot, expressions: &[String]) {
    report["capture"]["total_tasks"] = json!(state.total_tasks);
    report["capture"]["total_globals"] = json!(state.total_globals);
    report["capture"]["reasons"] = json!(state.reasons.iter().collect::<Vec<_>>());
    report["capture"]["incomplete"] = json!(!state.reasons.is_empty());
    let mut inspection_failed = state.selection_error.is_some();
    if let Some(error) = state.selection_error {
        report["diagnostics"]
            .as_array_mut()
            .unwrap()
            .push(diagnostic(
                "inspection_context",
                "inspection",
                &error.message,
            ));
    }
    let results = state
        .expressions
        .into_iter()
        .enumerate()
        .map(|(index, result)| {
            let (value, error) = match result {
                Ok(value) => (json!(CapturedValue::new(&value)), Value::Null),
                Err(error) => {
                    inspection_failed = true;
                    (
                        Value::Null,
                        diagnostic("expression_failed", "inspection", &error.message),
                    )
                }
            };
            json!({
                "expression": expressions[index], "task_id": state.selected_task,
                "frame": state.selected_frame, "value": value, "error": error,
            })
        })
        .collect::<Vec<_>>();
    report["expressions"] = json!(results);
    if inspection_failed && report["exit_code"] == 0 {
        report["exit_code"] = json!(3);
    }
    report["globals"] = json!(
        state
            .globals
            .iter()
            .map(|global| json!({
                "name":global.name, "value":CapturedValue::new(&global.value),
            }))
            .collect::<Vec<_>>()
    );
    report["tasks"] = json!(state.tasks.iter().map(task).collect::<Vec<_>>());
}

fn task(task: &DebugTaskSnapshot) -> Value {
    json!({
        "id": task.task.id, "name": task.task.name, "state": task.task.state.as_str(),
        "reason": task.reason, "total_frames": task.total_frames,
        "frames": task.frames.iter().map(frame).collect::<Vec<_>>(),
    })
}
fn frame(frame: &FailureFrame) -> Value {
    let location = frame.location.as_ref().map(|location| {
        json!({
            "source_id":location.source.get(), "line":location.line, "column":location.column,
        })
    });
    json!({
        "depth":frame.depth, "function":frame.function, "location":location,
        "reason":frame.reason, "truncated":frame.reason.is_some(),
        "bindings":frame.bindings.iter().map(binding).collect::<Vec<_>>(),
    })
}
fn binding(binding: &FailureBinding) -> Value {
    let kind = match binding.kind {
        fpas_bytecode::DebugBindingKind::Parameter => "parameter",
        fpas_bytecode::DebugBindingKind::Local => "local",
        fpas_bytecode::DebugBindingKind::Capture => "capture",
    };
    json!({
        "name":binding.name, "kind":kind, "scope_id":binding.scope_id,
        "value":CapturedValue::new(&binding.value),
    })
}
