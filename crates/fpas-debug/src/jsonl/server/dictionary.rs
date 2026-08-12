//! JSONL mapping for atomic dictionary structure mutation.

use serde_json::{Map, Value, json};

use super::{JsonlServer, ServerStatus};
use crate::evaluation::{parse_debug_assignment_target, parse_debug_expression};
use crate::jsonl::encode::{invalid_state, missing_argument, optional_u64_argument};
use crate::jsonl::protocol::{session_error, success};

impl JsonlServer {
    pub(super) fn insert_dictionary(
        &mut self,
        request_id: u64,
        command: &str,
        arguments: &Map<String, Value>,
    ) -> Vec<Value> {
        let request = match parse_request(
            request_id,
            command,
            arguments,
            &["key", "expression"],
            self.status,
        ) {
            Ok(request) => request,
            Err(response) => return vec![response],
        };
        let Some(session) = self.actor.session_mut() else {
            return vec![invalid_state(request_id, command, self.status)];
        };
        match session.insert_dictionary_entry_with_limits(
            &request.target,
            &request.expressions[0],
            &request.expressions[1],
            request.frame_id,
            request.limits,
        ) {
            Ok(result) => vec![success(request_id, command, result_body(result))],
            Err(error) => vec![session_error(request_id, command, error)],
        }
    }

    pub(super) fn remove_dictionary(
        &mut self,
        request_id: u64,
        command: &str,
        arguments: &Map<String, Value>,
    ) -> Vec<Value> {
        let request = match parse_request(request_id, command, arguments, &["key"], self.status) {
            Ok(request) => request,
            Err(response) => return vec![response],
        };
        let Some(session) = self.actor.session_mut() else {
            return vec![invalid_state(request_id, command, self.status)];
        };
        match session.remove_dictionary_entry_with_limits(
            &request.target,
            &request.expressions[0],
            request.frame_id,
            request.limits,
        ) {
            Ok(result) => vec![success(request_id, command, result_body(result))],
            Err(error) => vec![session_error(request_id, command, error)],
        }
    }

    pub(super) fn replace_dictionary_key(
        &mut self,
        request_id: u64,
        command: &str,
        arguments: &Map<String, Value>,
    ) -> Vec<Value> {
        let request = match parse_request(
            request_id,
            command,
            arguments,
            &["key", "new_key"],
            self.status,
        ) {
            Ok(request) => request,
            Err(response) => return vec![response],
        };
        let Some(session) = self.actor.session_mut() else {
            return vec![invalid_state(request_id, command, self.status)];
        };
        match session.replace_dictionary_key_with_limits(
            &request.target,
            &request.expressions[0],
            &request.expressions[1],
            request.frame_id,
            request.limits,
        ) {
            Ok(result) => vec![success(request_id, command, result_body(result))],
            Err(error) => vec![session_error(request_id, command, error)],
        }
    }
}

struct DictionaryRequest {
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
    status: ServerStatus,
) -> Result<DictionaryRequest, Value> {
    if status != ServerStatus::Stopped {
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
    Ok(DictionaryRequest {
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

fn result_body(result: fpas_vm::DebugDictionaryMutationResult) -> Value {
    let dictionary = result.dictionary;
    let mut body = Map::from_iter([
        ("result".to_string(), Value::String(dictionary.value)),
        ("type_name".to_string(), Value::String(dictionary.type_name)),
        (
            "variables_reference".to_string(),
            Value::from(dictionary.variables_reference),
        ),
        (
            "named_variables".to_string(),
            Value::from(dictionary.named_variables),
        ),
        (
            "indexed_variables".to_string(),
            Value::from(dictionary.indexed_variables),
        ),
    ]);
    if let Some(removed) = result.removed {
        body.insert("removed".to_string(), Value::String(removed));
    }
    if let Some(old_key) = result.old_key {
        body.insert("old_key".to_string(), Value::String(old_key));
    }
    if let Some(new_key) = result.new_key {
        body.insert("new_key".to_string(), Value::String(new_key));
    }
    Value::Object(body)
}
