//! DAP custom-request mapping for dictionary structure mutation.

use serde_json::{Value, json};

use super::DapServer;

impl DapServer {
    pub(super) fn insert_dictionary(
        &mut self,
        request_seq: u64,
        command: &str,
        arguments: &Value,
    ) -> Vec<Value> {
        let mut records = self.core_request(
            request_seq,
            command,
            "dictionary.insert",
            json!({
                "frame_id": arguments.get("frameId").cloned().unwrap_or(Value::Null),
                "target": arguments.get("target").cloned().unwrap_or(Value::Null),
                "key": arguments.get("key").cloned().unwrap_or(Value::Null),
                "expression": arguments.get("value").cloned().unwrap_or(Value::Null)
            }),
        );
        self.append_variables_invalidation(&mut records);
        records
    }

    pub(super) fn remove_dictionary(
        &mut self,
        request_seq: u64,
        command: &str,
        arguments: &Value,
    ) -> Vec<Value> {
        let mut records = self.core_request(
            request_seq,
            command,
            "dictionary.remove",
            json!({
                "frame_id": arguments.get("frameId").cloned().unwrap_or(Value::Null),
                "target": arguments.get("target").cloned().unwrap_or(Value::Null),
                "key": arguments.get("key").cloned().unwrap_or(Value::Null)
            }),
        );
        self.append_variables_invalidation(&mut records);
        records
    }

    pub(super) fn replace_dictionary_key(
        &mut self,
        request_seq: u64,
        command: &str,
        arguments: &Value,
    ) -> Vec<Value> {
        let mut records = self.core_request(
            request_seq,
            command,
            "dictionary.replace_key",
            json!({
                "frame_id": arguments.get("frameId").cloned().unwrap_or(Value::Null),
                "target": arguments.get("target").cloned().unwrap_or(Value::Null),
                "key": arguments.get("key").cloned().unwrap_or(Value::Null),
                "new_key": arguments.get("newKey").cloned().unwrap_or(Value::Null)
            }),
        );
        self.append_variables_invalidation(&mut records);
        records
    }
}
