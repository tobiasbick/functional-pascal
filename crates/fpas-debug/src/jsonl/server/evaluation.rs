//! Read-only stopped-frame evaluation request handling.

use serde_json::{Map, Value, json};

use super::{JsonlServer, ServerStatus};
use crate::evaluation::parse_debug_expression;
use crate::jsonl::encode::{invalid_state, missing_argument};
use crate::jsonl::protocol::{session_error, success};

impl JsonlServer {
    pub(super) fn evaluate(
        &mut self,
        request_id: u64,
        command: &str,
        arguments: &Map<String, Value>,
    ) -> Vec<Value> {
        if self.status != ServerStatus::Stopped {
            return vec![invalid_state(request_id, command, self.status)];
        }
        let Some(source) = arguments.get("expression").and_then(Value::as_str) else {
            return vec![missing_argument(request_id, command, "expression")];
        };
        let frame_id = arguments.get("frame_id").and_then(Value::as_u64);
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
        if arguments
            .get("async")
            .and_then(Value::as_bool)
            .unwrap_or(false)
        {
            let _ = session;
            self.pending_evaluation = Some((request_id, command.to_string()));
            self.actor.evaluate(expression, frame_id, limits);
            return Vec::new();
        }
        match session.evaluate_with_limits(&expression, frame_id, limits) {
            Ok(result) => vec![success(
                request_id,
                command,
                json!({
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
