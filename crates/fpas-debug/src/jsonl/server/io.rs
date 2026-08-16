//! JSONL mapping for queued debuggee input, EOF, and cancel.

use serde_json::{Map, Value, json};

use super::{JsonlServer, ServerStatus};
use crate::jsonl::encode::{invalid_state, missing_argument};
use crate::jsonl::protocol::{session_error, success};

impl JsonlServer {
    /// Queue one debuggee input line onto the session-owned channel.
    pub(super) fn push_debuggee_input(
        &mut self,
        request_id: u64,
        command: &str,
        arguments: &Map<String, Value>,
    ) -> Vec<Value> {
        if self.status != ServerStatus::Stopped {
            return vec![invalid_state(request_id, command, self.status)];
        }
        let Some(text) = arguments.get("text").and_then(Value::as_str) else {
            return vec![missing_argument(request_id, command, "text")];
        };
        let Some(session) = self.actor.session_mut() else {
            return vec![invalid_state(request_id, command, self.status)];
        };
        match session.push_debuggee_input(text) {
            Ok(result) => vec![success(
                request_id,
                command,
                json!({"bytes": result.bytes, "session_bytes": result.session_bytes}),
            )],
            Err(error) => vec![session_error(request_id, command, error)],
        }
    }

    /// Signal debuggee input EOF without mixing protocol stdin.
    pub(super) fn signal_debuggee_eof(&mut self, request_id: u64, command: &str) -> Vec<Value> {
        if self.status != ServerStatus::Stopped {
            return vec![invalid_state(request_id, command, self.status)];
        }
        let Some(session) = self.actor.session_mut() else {
            return vec![invalid_state(request_id, command, self.status)];
        };
        match session.signal_debuggee_eof() {
            Ok(()) => vec![success(request_id, command, json!({"eof": true}))],
            Err(error) => vec![session_error(request_id, command, error)],
        }
    }

    /// Drop unread queued debuggee input.
    pub(super) fn cancel_debuggee_input(&mut self, request_id: u64, command: &str) -> Vec<Value> {
        if self.status != ServerStatus::Stopped {
            return vec![invalid_state(request_id, command, self.status)];
        }
        let Some(session) = self.actor.session_mut() else {
            return vec![invalid_state(request_id, command, self.status)];
        };
        match session.cancel_debuggee_input() {
            Ok(()) => vec![success(request_id, command, json!({"cleared": true}))],
            Err(error) => vec![session_error(request_id, command, error)],
        }
    }
}
