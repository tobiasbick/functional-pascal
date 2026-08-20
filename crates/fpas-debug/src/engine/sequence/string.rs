//! JSONL mapping for string character replacement.

use serde_json::{Map, Value};

use super::*;
use crate::jsonl::protocol::{session_error, success};

impl DebugEngine {
    pub(in crate::engine) fn replace_string_character(
        &mut self,
        request_id: u64,
        command: &str,
        arguments: &Map<String, Value>,
    ) -> Vec<Value> {
        let request = match parse_request(
            request_id,
            command,
            arguments,
            &["index", "expression"],
            self.status,
        ) {
            Ok(request) => request,
            Err(response) => return vec![response],
        };
        let Some(session) = self.actor.session_mut() else {
            return vec![invalid_state(request_id, command, self.status)];
        };
        match session.replace_string_character_with_limits(
            &request.target,
            &request.expressions[0],
            &request.expressions[1],
            request.frame_id,
            request.limits,
        ) {
            Ok(result) => {
                let mut body = evaluation_body(result.string);
                body.insert("index".to_string(), Value::from(result.index));
                body.insert(
                    "old_character".to_string(),
                    Value::String(result.old_character),
                );
                body.insert(
                    "new_character".to_string(),
                    Value::String(result.new_character),
                );
                vec![success(request_id, command, Value::Object(body))]
            }
            Err(error) => vec![session_error(request_id, command, error)],
        }
    }
}
