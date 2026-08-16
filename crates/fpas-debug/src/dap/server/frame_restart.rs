//! DAP standard-request mapping for selected live-frame restart.

use serde_json::{Value, json};

use super::DapServer;

/// Translate restart metadata into DAP-friendly extension fields.
pub(super) fn response_body(command: &str, body: &Value) -> Option<Value> {
    (command == "restartFrame").then(|| {
        json!({
            "taskId": body.get("task_id"),
            "discardedFrames": body.get("discarded_frames")
        })
    })
}

impl DapServer {
    pub(super) fn restart_frame(
        &mut self,
        request_seq: u64,
        command: &str,
        arguments: &Value,
    ) -> Vec<Value> {
        let mut records = self.core_request(
            request_seq,
            command,
            "frame.restart",
            json!({
                "frame_id": arguments.get("frameId").cloned().unwrap_or(Value::Null)
            }),
        );
        if restart_succeeded(&records) && self.supports_invalidated_event {
            records.push(self.event("invalidated", json!({"areas":["stacks","variables"]})));
        }
        records
    }
}

fn restart_succeeded(records: &[Value]) -> bool {
    records.first().is_some_and(|record| {
        record.get("type").and_then(Value::as_str) == Some("response")
            && record.get("success").and_then(Value::as_bool) == Some(true)
    })
}
