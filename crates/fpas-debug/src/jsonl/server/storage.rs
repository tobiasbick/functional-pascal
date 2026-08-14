//! JSONL mapping for seeded empty-storage descendant initialization.

use serde_json::{Map, Value, json};

use super::{JsonlServer, ServerStatus};
use crate::evaluation::{parse_debug_assignment_target, parse_debug_expression};
use crate::jsonl::encode::{invalid_state, missing_argument, optional_u64_argument};
use crate::jsonl::protocol::{failure, session_error, success};

impl JsonlServer {
    pub(super) fn initialize_storage(
        &mut self,
        request_id: u64,
        command: &str,
        arguments: &Map<String, Value>,
    ) -> Vec<Value> {
        if self.status != ServerStatus::Stopped {
            return vec![invalid_state(request_id, command, self.status)];
        }
        if let Err(response) = require_only_keys(
            request_id,
            command,
            arguments,
            &["frame_id", "target", "initializer", "expression"],
        ) {
            return vec![response];
        }
        let request = match parse_request(request_id, command, arguments) {
            Ok(request) => request,
            Err(response) => return vec![response],
        };
        let Some(session) = self.actor.session_mut() else {
            return vec![invalid_state(request_id, command, self.status)];
        };
        match session.initialize_storage_with_limits(
            &request.target,
            &request.initializer,
            &request.expression,
            request.frame_id,
            request.limits,
        ) {
            Ok(result) => vec![success(
                request_id,
                command,
                json!({
                    "root": result.root,
                    "target": result.target,
                    "root_value": result.root_value,
                    "value": result.value.value,
                    "type": result.value.type_name,
                    "variables_reference": result.value.variables_reference,
                    "named_variables": result.value.named_variables,
                    "indexed_variables": result.value.indexed_variables
                }),
            )],
            Err(error) => vec![session_error(request_id, command, error)],
        }
    }
}

struct StorageRequest {
    target: fpas_vm::DebugAssignmentTarget,
    initializer: fpas_vm::DebugExpression,
    expression: fpas_vm::DebugExpression,
    frame_id: Option<u64>,
    limits: fpas_vm::DebugEvaluationLimits,
}

fn parse_request(
    request_id: u64,
    command: &str,
    arguments: &Map<String, Value>,
) -> Result<StorageRequest, Value> {
    let target_source = require_string(request_id, command, arguments, "target")?;
    let initializer_source = require_string(request_id, command, arguments, "initializer")?;
    let expression_source = require_string(request_id, command, arguments, "expression")?;
    let frame_id = optional_u64_argument(request_id, command, arguments, "frame_id")?;
    let limits = fpas_vm::DebugEvaluationLimits::default();
    let target = parse_debug_assignment_target(target_source, limits)
        .map_err(|error| parse_error(request_id, command, error))?;
    let initializer = parse_debug_expression(initializer_source, limits)
        .map_err(|error| parse_error(request_id, command, error))?;
    let expression = parse_debug_expression(expression_source, limits)
        .map_err(|error| parse_error(request_id, command, error))?;
    Ok(StorageRequest {
        target,
        initializer,
        expression,
        frame_id,
        limits,
    })
}

fn require_only_keys(
    request_id: u64,
    command: &str,
    arguments: &Map<String, Value>,
    allowed: &[&str],
) -> Result<(), Value> {
    let extra = arguments
        .keys()
        .find(|key| !allowed.contains(&key.as_str()));
    match extra {
        None => Ok(()),
        Some(name) => Err(failure(
            request_id,
            command,
            "invalid_request",
            format!("Command `{command}` does not accept argument `{name}`."),
            "Pass only `frame_id`, `target`, `initializer`, and `expression`.",
        )),
    }
}

fn require_string<'a>(
    request_id: u64,
    command: &str,
    arguments: &'a Map<String, Value>,
    name: &str,
) -> Result<&'a str, Value> {
    match arguments.get(name) {
        None => Err(missing_argument(request_id, command, name)),
        Some(Value::String(value)) => Ok(value.as_str()),
        Some(_) => Err(failure(
            request_id,
            command,
            "invalid_request",
            format!("Command `{command}` argument `{name}` must be a string."),
            "Pass one FPAS expression string.",
        )),
    }
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
