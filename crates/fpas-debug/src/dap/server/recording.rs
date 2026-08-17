//! DAP custom-request mapping for the recording envelope and capture log.

use serde_json::{Value, json};

use super::DapServer;

impl DapServer {
    pub(super) fn describe_recording(&mut self, request_seq: u64, command: &str) -> Vec<Value> {
        self.core_request(request_seq, command, "recording.describe", json!({}))
    }

    pub(super) fn start_recording(&mut self, request_seq: u64, command: &str) -> Vec<Value> {
        self.core_request(request_seq, command, "record", json!({}))
    }
}

/// Translate one recording custom-request result into DAP naming.
pub(super) fn response_body(command: &str, body: &Value) -> Option<Value> {
    match command {
        "fpas/recordingDescribe" => Some(json!({
            "version": body.get("version"),
            "bytecodeVersion": body.get("bytecode_version"),
            "program": body.get("program"),
            "sources": body.get("sources"),
            "capturing": body.get("capturing"),
            "events": body.get("events").and_then(Value::as_array).into_iter().flatten().map(dap_event).collect::<Vec<_>>(),
        })),
        "fpas/record" => Some(json!({
            "capturing": body.get("capturing"),
            "eventCount": body.get("event_count"),
        })),
        _ => None,
    }
}

fn dap_event(event: &Value) -> Value {
    match event.get("kind").and_then(Value::as_str) {
        Some("input") => json!({
            "kind": "input",
            "text": event.get("text"),
        }),
        _ => json!({
            "kind": "stop",
            "taskId": event.get("task_id"),
            "reason": event.get("reason"),
            "instruction": event.get("instruction"),
        }),
    }
}
