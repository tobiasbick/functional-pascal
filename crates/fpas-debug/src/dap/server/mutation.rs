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
        self.append_variables_invalidation(&mut records);
        records
    }

    /// Maps standard DAP `setExpression` arguments to the shared JSONL operation.
    pub(super) fn set_expression(
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
                "Non-default DAP value formatting is not supported for setExpression.",
            )];
        }
        let mut records = self.core_request(
            request_seq,
            command,
            "expression.set",
            json!({
                "target": arguments.get("expression").cloned().unwrap_or(Value::Null),
                "expression": arguments.get("value").cloned().unwrap_or(Value::Null),
                "frame_id": arguments.get("frameId").cloned().unwrap_or(Value::Null)
            }),
        );
        self.append_variables_invalidation(&mut records);
        records
    }

    pub(super) fn append_variables_invalidation(&mut self, records: &mut Vec<Value>) {
        let succeeded = records.first().is_some_and(|record| {
            record.get("type").and_then(Value::as_str) == Some("response")
                && record.get("success").and_then(Value::as_bool) == Some(true)
        });
        if succeeded && self.supports_invalidated_event {
            records.push(self.event("invalidated", json!({"areas":["variables"]})));
        }
    }
}
