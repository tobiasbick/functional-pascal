//! DAP mapping for queued debuggee input, EOF, and cancel.

use serde_json::{Value, json};

use super::DapServer;

pub(super) fn response_body(command: &str, body: &Value) -> Option<Value> {
    match command {
        "fpas/input" => Some(json!({
            "bytes": body.get("bytes"),
            "sessionBytes": body.get("session_bytes")
        })),
        "fpas/eof" => Some(json!({"eof": body.get("eof")})),
        "fpas/cancelInput" => Some(json!({"cleared": body.get("cleared")})),
        _ => None,
    }
}

impl DapServer {
    pub(super) fn push_debuggee_input(
        &mut self,
        request_seq: u64,
        command: &str,
        arguments: &Value,
    ) -> Vec<Value> {
        self.core_request(
            request_seq,
            command,
            "io.input",
            json!({"text": arguments.get("text").cloned().unwrap_or(Value::Null)}),
        )
    }

    pub(super) fn signal_debuggee_eof(&mut self, request_seq: u64, command: &str) -> Vec<Value> {
        self.core_request(request_seq, command, "io.eof", json!({}))
    }

    pub(super) fn cancel_debuggee_input(&mut self, request_seq: u64, command: &str) -> Vec<Value> {
        self.core_request(request_seq, command, "io.cancel", json!({}))
    }
}
