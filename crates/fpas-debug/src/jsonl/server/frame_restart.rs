//! JSONL mapping for selected live-frame restart.

use serde_json::{Map, Value, json};

use super::{JsonlServer, ServerStatus};
use crate::jsonl::encode::{frame_body, invalid_state, missing_argument};
use crate::jsonl::protocol::{session_error, success};

impl JsonlServer {
    pub(super) fn restart_frame(
        &mut self,
        request_id: u64,
        command: &str,
        arguments: &Map<String, Value>,
    ) -> Vec<Value> {
        if self.status != ServerStatus::Stopped {
            return vec![invalid_state(request_id, command, self.status)];
        }
        let Some(frame_id) = arguments.get("frame_id").and_then(Value::as_u64) else {
            return vec![missing_argument(request_id, command, "frame_id")];
        };
        let Some(session) = self.actor.session_mut() else {
            return vec![invalid_state(request_id, command, self.status)];
        };
        match session.restart_frame(frame_id) {
            Ok(result) => vec![success(
                request_id,
                command,
                json!({
                    "task_id": result.task_id,
                    "frame": frame_body(&result.frame),
                    "discarded_frames": result.discarded_frames
                }),
            )],
            Err(error) => vec![session_error(request_id, command, error)],
        }
    }
}
