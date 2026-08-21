//! DAP standard-request mapping for selected live-frame restart.

use serde_json::{Value, json};

use super::DapServer;
use super::args;
use crate::engine::{DebugOp, ResponseBody};

/// Translate restart metadata into DAP-friendly extension fields.
pub(super) fn response_body(command: &str, body: &ResponseBody) -> Option<Value> {
    if command != "restartFrame" {
        return None;
    }
    let ResponseBody::FrameRestart(result) = body else {
        return None;
    };
    Some(json!({
        "taskId": result.task_id,
        "discardedFrames": result.discarded_frames
    }))
}

impl DapServer {
    pub(super) fn restart_frame(
        &mut self,
        request_seq: u64,
        command: &str,
        arguments: &Value,
    ) -> Vec<Value> {
        match args::required_u64(arguments, "frameId") {
            Ok(frame_id) => {
                let mut records =
                    self.core_request(request_seq, command, DebugOp::FrameRestart { frame_id });
                if restart_succeeded(&records) && self.supports_invalidated_event {
                    records
                        .push(self.event("invalidated", json!({"areas":["stacks","variables"]})));
                }
                records
            }
            Err(message) => vec![self.failure(request_seq, command, &message)],
        }
    }
}

fn restart_succeeded(records: &[Value]) -> bool {
    records.first().is_some_and(|record| {
        record.get("type").and_then(Value::as_str) == Some("response")
            && record.get("success").and_then(Value::as_bool) == Some(true)
    })
}
