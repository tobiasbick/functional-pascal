//! DAP custom-request mapping for sequence structure mutation.

use serde_json::{Value, json};

use super::DapServer;

impl DapServer {
    pub(super) fn insert_array(
        &mut self,
        request_seq: u64,
        command: &str,
        arguments: &Value,
    ) -> Vec<Value> {
        self.sequence_request(request_seq, command, "array.insert", arguments, true)
    }

    pub(super) fn remove_array(
        &mut self,
        request_seq: u64,
        command: &str,
        arguments: &Value,
    ) -> Vec<Value> {
        self.sequence_request(request_seq, command, "array.remove", arguments, false)
    }

    pub(super) fn replace_string_character(
        &mut self,
        request_seq: u64,
        command: &str,
        arguments: &Value,
    ) -> Vec<Value> {
        self.sequence_request(
            request_seq,
            command,
            "string.replace_character",
            arguments,
            true,
        )
    }

    fn sequence_request(
        &mut self,
        request_seq: u64,
        command: &str,
        core_command: &str,
        arguments: &Value,
        includes_expression: bool,
    ) -> Vec<Value> {
        let mut body = json!({
            "frame_id": arguments.get("frameId").cloned().unwrap_or(Value::Null),
            "target": arguments.get("target").cloned().unwrap_or(Value::Null),
            "index": arguments.get("index").cloned().unwrap_or(Value::Null)
        });
        if includes_expression {
            body["expression"] = arguments.get("value").cloned().unwrap_or(Value::Null);
        }
        let mut records = self.core_request(request_seq, command, core_command, body);
        self.append_variables_invalidation(&mut records);
        records
    }
}
