//! JSONL mapping for rejected instruction-pointer changes.

use serde_json::{Map, Value};

use super::{JsonlServer, ServerStatus};
use crate::jsonl::encode::invalid_state;
use crate::jsonl::protocol::session_error;

impl JsonlServer {
    pub(super) fn set_instruction(
        &mut self,
        request_id: u64,
        command: &str,
        arguments: &Map<String, Value>,
    ) -> Vec<Value> {
        if self.status != ServerStatus::Stopped {
            return vec![invalid_state(request_id, command, self.status)];
        }
        let frame_id = match arguments.get("frame_id") {
            None | Some(Value::Null) => None,
            Some(value) => match value.as_u64() {
                Some(frame_id) => Some(frame_id),
                None => {
                    return vec![crate::jsonl::encode::missing_argument(
                        request_id, command, "frame_id",
                    )];
                }
            },
        };
        let instruction = match arguments.get("instruction") {
            None | Some(Value::Null) => None,
            Some(value) => match value.as_u64().and_then(|value| u32::try_from(value).ok()) {
                Some(instruction) => Some(instruction),
                None => {
                    return vec![crate::jsonl::encode::missing_argument(
                        request_id,
                        command,
                        "instruction",
                    )];
                }
            },
        };
        let Some(session) = self.actor.session_mut() else {
            return vec![invalid_state(request_id, command, self.status)];
        };
        vec![session_error(
            request_id,
            command,
            session.set_instruction(frame_id, instruction),
        )]
    }
}
