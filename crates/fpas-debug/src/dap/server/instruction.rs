//! DAP mapping for rejected `goto` / `gotoTargets` instruction changes.

use serde_json::{Value, json};

use super::DapServer;

impl DapServer {
    pub(super) fn set_instruction(
        &mut self,
        request_seq: u64,
        command: &str,
        arguments: &Value,
    ) -> Vec<Value> {
        let frame_id = arguments.get("frameId").cloned().unwrap_or(Value::Null);
        let instruction = arguments
            .get("targetId")
            .or_else(|| arguments.get("instruction"))
            .cloned()
            .unwrap_or(json!(0));
        self.core_request(
            request_seq,
            command,
            "instruction.set",
            json!({
                "frame_id": frame_id,
                "instruction": instruction
            }),
        )
    }
}
