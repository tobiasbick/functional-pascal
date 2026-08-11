//! DAP `setVariable` mapping and client-negotiated variable invalidation.

use serde_json::{Value, json};

use super::DapServer;

impl DapServer {
    pub(super) fn set_variable(
        &mut self,
        request_seq: u64,
        command: &str,
        arguments: &Value,
    ) -> Vec<Value> {
        if arguments
            .get("format")
            .is_some_and(|format| !format.is_null() && format != &json!({}))
        {
            return vec![self.failure(
                request_seq,
                command,
                "Non-default DAP value formatting is not supported for setVariable.",
            )];
        }
        let mut records = self.core_request(
            request_seq,
            command,
            "variable.set",
            json!({
                "variables_reference": arguments
                    .get("variablesReference")
                    .cloned()
                    .unwrap_or(Value::Null),
                "name": arguments.get("name").cloned().unwrap_or(Value::Null),
                "expression": arguments.get("value").cloned().unwrap_or(Value::Null)
            }),
        );
        let succeeded = records.first().is_some_and(|record| {
            record.get("type").and_then(Value::as_str) == Some("response")
                && record.get("success").and_then(Value::as_bool) == Some(true)
        });
        if succeeded && self.supports_invalidated_event {
            records.push(self.event("invalidated", json!({"areas":["variables"]})));
        }
        records
    }
}
