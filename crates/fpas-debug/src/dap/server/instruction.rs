//! DAP mapping for rejected `goto` / `gotoTargets` instruction changes.

use serde_json::Value;

use super::DapServer;
use super::args;
use crate::engine::DebugOp;

impl DapServer {
    pub(super) fn set_instruction(
        &mut self,
        request_seq: u64,
        command: &str,
        arguments: &Value,
    ) -> Vec<Value> {
        let frame_id = match args::optional_u64(arguments, "frameId") {
            Ok(frame_id) => frame_id,
            Err(message) => return vec![self.failure(request_seq, command, &message)],
        };
        let instruction = arguments
            .get("targetId")
            .or_else(|| arguments.get("instruction"))
            .and_then(Value::as_u64)
            .and_then(|value| u32::try_from(value).ok())
            .or(Some(0));
        self.core_request(
            request_seq,
            command,
            DebugOp::InstructionSet {
                frame_id,
                instruction,
            },
        )
    }
}
