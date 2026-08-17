//! DAP custom-request mapping for the recording envelope.

use serde_json::{Value, json};

use super::DapServer;

impl DapServer {
    pub(super) fn describe_recording(&mut self, request_seq: u64, command: &str) -> Vec<Value> {
        self.core_request(request_seq, command, "recording.describe", json!({}))
    }
}

/// Translate one recording custom-request result into DAP naming.
pub(super) fn response_body(command: &str, body: &Value) -> Option<Value> {
    if command != "fpas/recordingDescribe" {
        return None;
    }
    Some(json!({
        "version": body.get("version"),
        "bytecodeVersion": body.get("bytecode_version"),
        "program": body.get("program"),
        "sources": body.get("sources"),
    }))
}
