//! Protocol mapping for atomic stopped-state variable mutation.

use serde_json::{Map, Value, json};

use super::{DebugEngine, DebugStatus};
use crate::evaluation::{parse_debug_assignment_target, parse_debug_expression};
use crate::jsonl::encode::{invalid_state, missing_argument, optional_u64_argument};
use crate::jsonl::protocol::{session_error, success};

impl DebugEngine {
    pub(super) fn set_variable(
        &mut self,
        request_id: u64,
        command: &str,
        arguments: &Map<String, Value>,
    ) -> Vec<Value> {
        if self.status != DebugStatus::Stopped {
            return vec![invalid_state(request_id, command, self.status)];
        }
        let Some(variables_reference) =
            arguments.get("variables_reference").and_then(Value::as_u64)
        else {
            return vec![missing_argument(request_id, command, "variables_reference")];
        };
        let Some(name) = arguments.get("name").and_then(Value::as_str) else {
            return vec![missing_argument(request_id, command, "name")];
        };
        let Some(source) = arguments.get("expression").and_then(Value::as_str) else {
            return vec![missing_argument(request_id, command, "expression")];
        };
        let limits = fpas_vm::DebugEvaluationLimits::default();
        let expression = match parse_debug_expression(source, limits) {
            Ok(expression) => expression,
            Err(error) => {
                return vec![json!({
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
                })];
            }
        };
        let Some(session) = self.actor.session_mut() else {
            return vec![invalid_state(request_id, command, self.status)];
        };
        match session.set_variable_with_limits(variables_reference, name, &expression, limits) {
            Ok(result) => vec![success(request_id, command, result_body(result))],
            Err(error) => vec![session_error(request_id, command, error)],
        }
    }

    /// Replace one textual mutable target at the current stop.
    pub(super) fn set_expression(
        &mut self,
        request_id: u64,
        command: &str,
        arguments: &Map<String, Value>,
    ) -> Vec<Value> {
        if self.status != DebugStatus::Stopped {
            return vec![invalid_state(request_id, command, self.status)];
        }
        let Some(target_source) = arguments.get("target").and_then(Value::as_str) else {
            return vec![missing_argument(request_id, command, "target")];
        };
        let Some(replacement_source) = arguments.get("expression").and_then(Value::as_str) else {
            return vec![missing_argument(request_id, command, "expression")];
        };
        let frame_id = match optional_u64_argument(request_id, command, arguments, "frame_id") {
            Ok(frame_id) => frame_id,
            Err(response) => return vec![response],
        };
        let limits = fpas_vm::DebugEvaluationLimits::default();
        let target = match parse_debug_assignment_target(target_source, limits) {
            Ok(target) => target,
            Err(error) => return vec![parse_error(request_id, command, error)],
        };
        let replacement = match parse_debug_expression(replacement_source, limits) {
            Ok(expression) => expression,
            Err(error) => return vec![parse_error(request_id, command, error)],
        };
        let Some(session) = self.actor.session_mut() else {
            return vec![invalid_state(request_id, command, self.status)];
        };
        match session.set_expression_with_limits(&target, &replacement, frame_id, limits) {
            Ok(result) => vec![success(request_id, command, result_body(result))],
            Err(error) => vec![session_error(request_id, command, error)],
        }
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

fn result_body(result: fpas_vm::DebugEvaluateResult) -> Value {
    json!({
        "result": result.value,
        "type_name": result.type_name,
        "variables_reference": result.variables_reference,
        "named_variables": result.named_variables,
        "indexed_variables": result.indexed_variables
    })
}
