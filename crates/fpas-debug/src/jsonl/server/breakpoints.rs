//! Logical breakpoint configuration for the JSONL server.

use serde_json::{Map, Value, json};

use super::{JsonlServer, ServerStatus};
use crate::breakpoints::BreakpointPolicy;
use crate::jsonl::encode::{breakpoint_body, invalid_state, missing_argument};
use crate::jsonl::protocol::{event, session_error, success};

impl JsonlServer {
    pub(super) fn set_breakpoint(
        &mut self,
        request_id: u64,
        command: &str,
        arguments: &Map<String, Value>,
    ) -> Vec<Value> {
        if !matches!(
            self.status,
            ServerStatus::Initialized | ServerStatus::Stopped
        ) {
            return vec![invalid_state(request_id, command, self.status)];
        }
        let Some(source) = arguments.get("source").and_then(Value::as_str) else {
            return vec![missing_argument(request_id, command, "source")];
        };
        let Some(line) = arguments
            .get("line")
            .and_then(Value::as_u64)
            .and_then(|line| u32::try_from(line).ok())
            .filter(|line| *line > 0)
        else {
            return vec![missing_argument(request_id, command, "line")];
        };
        let column = arguments
            .get("column")
            .and_then(Value::as_u64)
            .and_then(|column| u32::try_from(column).ok());
        let policy = match BreakpointPolicy::parse(
            arguments.get("condition").and_then(Value::as_str),
            arguments.get("hit_condition").and_then(Value::as_str),
            arguments.get("log_message").and_then(Value::as_str),
        ) {
            Ok(policy) => policy,
            Err(error) => {
                return vec![success(
                    request_id,
                    command,
                    json!({
                        "verified": false,
                        "message": format!("{} Help: {}", error.message, error.hint),
                        "error_code": error.code,
                        "error_offset": error.offset,
                        "error_length": error.length,
                        "requested": {"source": source, "line": line, "column": column}
                    }),
                )];
            }
        };
        let breakpoint = match self.actor.session_mut().map(|session| {
            session.set_breakpoint(fpas_vm::SourceBreakpoint {
                source: source.to_string(),
                line,
                column,
            })
        }) {
            Some(Ok(breakpoint)) => breakpoint,
            Some(Err(error)) => return vec![session_error(request_id, command, error)],
            None => return vec![invalid_state(request_id, command, self.status)],
        };
        self.breakpoint_policies.insert(breakpoint.id, policy);
        let body = breakpoint_body(&breakpoint);
        vec![
            success(request_id, command, body.clone()),
            event("breakpoint", body),
        ]
    }

    pub(super) fn clear_breakpoint(
        &mut self,
        request_id: u64,
        command: &str,
        arguments: &Map<String, Value>,
    ) -> Vec<Value> {
        if !matches!(
            self.status,
            ServerStatus::Initialized | ServerStatus::Stopped
        ) {
            return vec![invalid_state(request_id, command, self.status)];
        }
        let Some(id) = arguments.get("breakpoint_id").and_then(Value::as_u64) else {
            return vec![missing_argument(request_id, command, "breakpoint_id")];
        };
        self.breakpoint_policies.remove(&id);
        match self
            .actor
            .session_mut()
            .map(|session| session.clear_breakpoint(id))
        {
            Some(Ok(())) => vec![success(request_id, command, json!({"breakpoint_id": id}))],
            Some(Err(error)) => vec![session_error(request_id, command, error)],
            None => vec![invalid_state(request_id, command, self.status)],
        }
    }
}
