//! JSONL mapping for array insertion and removal.

use serde_json::{Map, Value};

use super::*;
use crate::jsonl::protocol::{session_error, success};

impl DebugEngine {
    pub(in crate::engine) fn insert_array(
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
        match session.insert_array_element_with_limits(
            &request.target,
            &request.expressions[0],
            &request.expressions[1],
            request.frame_id,
            request.limits,
        ) {
            Ok(result) => vec![success(request_id, command, array_result_body(result))],
            Err(error) => vec![session_error(request_id, command, error)],
        }
    }

    pub(in crate::engine) fn remove_array(
        &mut self,
        request_id: u64,
        command: &str,
        arguments: &Map<String, Value>,
    ) -> Vec<Value> {
        let request = match parse_request(request_id, command, arguments, &["index"], self.status) {
            Ok(request) => request,
            Err(response) => return vec![response],
        };
        let Some(session) = self.actor.session_mut() else {
            return vec![invalid_state(request_id, command, self.status)];
        };
        match session.remove_array_element_with_limits(
            &request.target,
            &request.expressions[0],
            request.frame_id,
            request.limits,
        ) {
            Ok(result) => vec![success(request_id, command, array_result_body(result))],
            Err(error) => vec![session_error(request_id, command, error)],
        }
    }
}

fn array_result_body(result: fpas_vm::DebugArrayMutationResult) -> Value {
    let mut body = evaluation_body(result.array);
    body.insert("index".to_string(), Value::from(result.index));
    if let Some(removed) = result.removed {
        body.insert("removed".to_string(), Value::String(removed));
    }
    Value::Object(body)
}
