//! JSONL mapping for protocol-neutral forced return.

use serde_json::{Map, Value, json};

use super::{JsonlServer, ServerStatus};
use crate::evaluation::parse_debug_expression;
use crate::jsonl::encode::{frame_body, invalid_state, missing_argument};
use crate::jsonl::protocol::{failure, session_error, success};

impl JsonlServer {
    pub(super) fn force_return(
        &mut self,
        request_id: u64,
        command: &str,
        arguments: &Map<String, Value>,
    ) -> Vec<Value> {
        if self.status != ServerStatus::Stopped {
            return vec![invalid_state(request_id, command, self.status)];
        }
        let Some(frame_id) = arguments.get("frame_id").and_then(Value::as_u64) else {
            return vec![missing_argument(request_id, command, "frame_id")];
        };
        let expression_source = match arguments.get("expression") {
            None | Some(Value::Null) => None,
            Some(Value::String(source)) => Some(source.as_str()),
            Some(_) => {
                return vec![failure(
                    request_id,
                    command,
                    "invalid_request",
                    "Command `frame.return` argument `expression` must be a string when present.",
                    "Omit `expression` for procedures, or pass one FPAS expression string for functions.",
                )];
            }
        };
        let limits = fpas_vm::DebugEvaluationLimits::default();
        let expression = match expression_source {
            Some(source) => match parse_debug_expression(source, limits) {
                Ok(expression) => Some(expression),
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
            },
            None => None,
        };
        let Some(session) = self.actor.session_mut() else {
            return vec![invalid_state(request_id, command, self.status)];
        };
        match session.force_return_with_limits(frame_id, expression.as_ref(), limits) {
            Ok(result) => vec![success(
                request_id,
                command,
                json!({
                    "task_id": result.task_id,
                    "result": result.value,
                    "type_name": result.type_name,
                    "variables_reference": result.variables_reference,
                    "named_variables": result.named_variables,
                    "indexed_variables": result.indexed_variables,
                    "unwound_frames": result.unwound_frames,
                    "frame": frame_body(&result.frame)
                }),
            )],
            Err(error) => vec![session_error(request_id, command, error)],
        }
    }
}
