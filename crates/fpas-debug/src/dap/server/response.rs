//! DAP response bodies from typed debug engine records.

use serde_json::{Value, json};

use super::{
    breakpoints, completed_result, data_breakpoints, exceptions, forced_return, frame_restart, io,
    live_image, location, mutation, recording, storage, task_control, values, variant,
};
use crate::engine::ResponseBody;

pub(super) fn dap_body(command: &str, body: ResponseBody) -> Value {
    if let Some(result) = breakpoints::response_body(command, &body) {
        return result;
    }
    if let Some(result) = data_breakpoints::response_body(command, &body) {
        return result;
    }
    if let Some(result) = exceptions::response_body(command) {
        return result;
    }
    if let Some(result) = forced_return::response_body(command, &body) {
        return result;
    }
    if let Some(result) = frame_restart::response_body(command, &body) {
        return result;
    }
    if let Some(result) = completed_result::response_body(command, &body) {
        return result;
    }
    if let Some(result) = task_control::response_body(command, &body) {
        return result;
    }
    if let Some(result) = variant::response_body(command, &body) {
        return result;
    }
    if let Some(result) = storage::response_body(command, &body) {
        return result;
    }
    if let Some(result) = io::response_body(command, &body) {
        return result;
    }
    if let Some(result) = mutation::custom_response_body(command, &body) {
        return result;
    }
    if let Some(result) = location::response_body(command, &body) {
        return result;
    }
    if let Some(result) = recording::response_body(command, &body) {
        return result;
    }
    if let Some(result) = live_image::response_body(command, &body) {
        return result;
    }
    match (command, body) {
        ("stackTrace", ResponseBody::Stack { frames, total, .. }) => json!({
            "stackFrames": frames.iter().map(values::frame_json).collect::<Vec<_>>(),
            "totalFrames": total
        }),
        ("scopes", ResponseBody::Scopes { scopes }) => json!({
            "scopes": scopes.iter().map(|scope| json!({
                "name": scope.name,
                "variablesReference": scope.variables_reference,
                "namedVariables": scope.named_variables,
                "expensive": scope.expensive
            })).collect::<Vec<_>>()
        }),
        ("variables", ResponseBody::Variables { variables, .. }) => json!({
            "variables": variables.iter().map(|variable| json!({
                "name": variable.name,
                "value": variable.value,
                "type": variable.type_name,
                "variablesReference": variable.variables_reference,
                "namedVariables": variable.named_variables,
                "indexedVariables": variable.indexed_variables
            })).collect::<Vec<_>>()
        }),
        ("evaluate", ResponseBody::Evaluate(result)) => values::evaluate_json(&result),
        ("setVariable" | "setExpression", ResponseBody::Evaluate(result)) => {
            Value::Object(values::variable_value_json(&result))
        }
        ("continue", _) => json!({"allThreadsContinued": true}),
        ("cancel", ResponseBody::Cancelled { cancelled }) => json!({"cancelled": cancelled}),
        _ => json!({}),
    }
}
