//! JSONL mapping for replacement of one retained completed task result.

use serde_json::{Map, Value, json};

use super::{DebugEngine, DebugStatus};
use crate::evaluation::parse_debug_expression;
use crate::jsonl::encode::{invalid_state, missing_argument, optional_u64_argument};
use crate::jsonl::protocol::{failure, session_error, success};

impl DebugEngine {
    pub(super) fn replace_completed_task_result(
        &mut self,
        request_id: u64,
        command: &str,
        arguments: &Map<String, Value>,
    ) -> Vec<Value> {
        if self.status != DebugStatus::Stopped {
            return vec![invalid_state(request_id, command, self.status)];
        }
        let task_id = match required_task_id(request_id, command, arguments) {
            Ok(task_id) => task_id,
            Err(error) => return vec![error],
        };
        let frame_id = match optional_u64_argument(request_id, command, arguments, "frame_id") {
            Ok(frame_id) => frame_id,
            Err(error) => return vec![error],
        };
        let limits = fpas_vm::DebugEvaluationLimits::default();
        let expression = match parse_expression(request_id, command, arguments, limits) {
            Ok(expression) => expression,
            Err(error) => return vec![error],
        };
        let Some(session) = self.actor.session_mut() else {
            return vec![invalid_state(request_id, command, self.status)];
        };
        match session.replace_completed_task_result_with_limits(
            task_id,
            frame_id,
            expression.as_ref(),
            limits,
        ) {
            Ok(result) => vec![success(
                request_id,
                command,
                json!({
                    "task_id": result.task_id,
                    "result": result.value,
                    "type_name": result.type_name,
                    "variables_reference": result.variables_reference,
                    "named_variables": result.named_variables,
                    "indexed_variables": result.indexed_variables
                }),
            )],
            Err(error) => vec![session_error(request_id, command, error)],
        }
    }
}

fn required_task_id(
    request_id: u64,
    command: &str,
    arguments: &Map<String, Value>,
) -> Result<u64, Value> {
    match arguments.get("task_id") {
        None | Some(Value::Null) => Err(missing_argument(request_id, command, "task_id")),
        Some(value) => value.as_u64().ok_or_else(|| {
            failure(
                request_id,
                command,
                "invalid_request",
                format!("Command `{command}` argument `task_id` must be a non-negative integer."),
                "Pass a task ID returned by `tasks` as `task_id`.",
            )
        }),
    }
}

fn parse_expression(
    request_id: u64,
    command: &str,
    arguments: &Map<String, Value>,
    limits: fpas_vm::DebugEvaluationLimits,
) -> Result<Option<fpas_vm::DebugExpression>, Value> {
    let source = match arguments.get("expression") {
        None | Some(Value::Null) => return Ok(None),
        Some(Value::String(source)) => source,
        Some(_) => {
            return Err(failure(
                request_id,
                command,
                "invalid_request",
                "Command `task.result.replace` argument `expression` must be a string when present.",
                "Omit `expression` for procedure tasks, or pass one FPAS expression string for function tasks.",
            ));
        }
    };
    parse_debug_expression(source, limits)
        .map(Some)
        .map_err(|error| {
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
        })
}
