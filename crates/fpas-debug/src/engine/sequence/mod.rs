//! Shared JSONL parsing and result encoding for sequence structure mutation.

mod array;
mod string;

use serde_json::{Map, Value, json};

use super::{DebugEngine, DebugStatus};
use crate::evaluation::{parse_debug_assignment_target, parse_debug_expression};
use crate::jsonl::encode::{invalid_state, missing_argument, optional_u64_argument};

struct SequenceRequest {
    target: fpas_vm::DebugAssignmentTarget,
    expressions: Vec<fpas_vm::DebugExpression>,
    frame_id: Option<u64>,
    limits: fpas_vm::DebugEvaluationLimits,
}

fn parse_request(
    request_id: u64,
    command: &str,
    arguments: &Map<String, Value>,
    expression_names: &[&str],
    status: DebugStatus,
) -> Result<SequenceRequest, Value> {
    if status != DebugStatus::Stopped {
        return Err(invalid_state(request_id, command, status));
    }
    let Some(target_source) = arguments.get("target").and_then(Value::as_str) else {
        return Err(missing_argument(request_id, command, "target"));
    };
    let frame_id = optional_u64_argument(request_id, command, arguments, "frame_id")?;
    let limits = fpas_vm::DebugEvaluationLimits::default();
    let target = parse_debug_assignment_target(target_source, limits)
        .map_err(|error| parse_error(request_id, command, error))?;
    let mut expressions = Vec::with_capacity(expression_names.len());
    for name in expression_names {
        let Some(source) = arguments.get(*name).and_then(Value::as_str) else {
            return Err(missing_argument(request_id, command, name));
        };
        expressions.push(
            parse_debug_expression(source, limits)
                .map_err(|error| parse_error(request_id, command, error))?,
        );
    }
    Ok(SequenceRequest {
        target,
        expressions,
        frame_id,
        limits,
    })
}

fn parse_error(
    request_id: u64,
    command: &str,
    error: crate::evaluation::EvaluationParseError,
) -> Value {
    json!({
        "type": "response",
        "request_id": request_id,
        "command": command,
        "success": false,
        "error": {
            "code": error.code,
            "message": error.message,
            "help": error.hint,
            "offset": error.offset,
            "length": error.length
        }
    })
}

fn evaluation_body(result: fpas_vm::DebugEvaluateResult) -> Map<String, Value> {
    Map::from_iter([
        ("result".to_string(), Value::String(result.value)),
        ("type_name".to_string(), Value::String(result.type_name)),
        (
            "variables_reference".to_string(),
            Value::from(result.variables_reference),
        ),
        (
            "named_variables".to_string(),
            Value::from(result.named_variables),
        ),
        (
            "indexed_variables".to_string(),
            Value::from(result.indexed_variables),
        ),
    ])
}
