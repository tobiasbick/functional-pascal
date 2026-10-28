//! Shared DAP JSON for engine inspection values.

use serde_json::{Value, json};

/// DAP stack frame object.
pub(super) fn frame_json(frame: &fpas_runtime::execution::DebugFrame) -> Value {
    json!({
        "id": frame.id,
        "name": frame.name,
        "source": {"path": frame.location.as_ref().map(|location| location.source.as_str())},
        "line": frame.location.as_ref().map_or(1, |location| location.line),
        "column": frame.location.as_ref().map_or(1, |location| location.column)
    })
}

/// DAP evaluate / hover body.
pub(super) fn evaluate_json(result: &fpas_runtime::execution::DebugEvaluateResult) -> Value {
    json!({
        "result": result.value,
        "type": result.type_name,
        "variablesReference": result.variables_reference,
        "namedVariables": result.named_variables,
        "indexedVariables": result.indexed_variables
    })
}

/// DAP setVariable / setExpression / mutation body.
pub(super) fn variable_value_json(
    result: &fpas_runtime::execution::DebugEvaluateResult,
) -> serde_json::Map<String, Value> {
    serde_json::Map::from_iter([
        ("value".into(), Value::String(result.value.clone())),
        ("type".into(), Value::String(result.type_name.clone())),
        (
            "variablesReference".into(),
            Value::from(result.variables_reference),
        ),
        ("namedVariables".into(), Value::from(result.named_variables)),
        (
            "indexedVariables".into(),
            Value::from(result.indexed_variables),
        ),
    ])
}

/// DAP location identity object.
pub(super) fn identity_json(identity: fpas_runtime::execution::DebugDataLocationIdentity) -> Value {
    match identity {
        fpas_runtime::execution::DebugDataLocationIdentity::Global { index } => {
            json!({"index": index})
        }
        fpas_runtime::execution::DebugDataLocationIdentity::FrameRegister {
            task_id,
            function,
            register,
        } => json!({
            "taskId": task_id,
            "function": function,
            "register": register
        }),
    }
}
