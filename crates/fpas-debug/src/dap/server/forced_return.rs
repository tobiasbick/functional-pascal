//! DAP custom-request mapping for forced return.

use serde_json::{Value, json};

use super::DapServer;

/// Translate one forced-return result into DAP naming.
pub(super) fn response_body(command: &str, body: &Value) -> Option<Value> {
    if command != "fpas/forceReturn" {
        return None;
    }
    let frame = body.get("frame").cloned().unwrap_or(Value::Null);
    let frame = (!frame.is_null()).then(|| {
        json!({
            "id": frame.get("frame_id"),
            "name": frame.get("name"),
            "source": {"path": frame.pointer("/location/source")},
            "line": frame.pointer("/location/line").unwrap_or(&json!(1)),
            "column": frame.pointer("/location/column").unwrap_or(&json!(1))
        })
    });
    Some(json!({
        "value": body.get("result"),
        "type": body.get("type_name"),
        "variablesReference": body.get("variables_reference"),
        "namedVariables": body.get("named_variables"),
        "indexedVariables": body.get("indexed_variables"),
        "unwoundFrames": body.get("unwound_frames"),
        "taskId": body.get("task_id"),
        "frame": frame,
        "terminated": body.get("terminated")
    }))
}

impl DapServer {
    pub(super) fn force_return(
        &mut self,
        request_seq: u64,
        command: &str,
        arguments: &Value,
    ) -> Vec<Value> {
        let mut records = self.core_request(
            request_seq,
            command,
            "frame.return",
            json!({
                "frame_id": arguments.get("frameId").cloned().unwrap_or(Value::Null),
                "expression": arguments.get("expression").cloned().unwrap_or(Value::Null)
            }),
        );
        if records.first().is_some_and(|record| {
            record.get("type").and_then(Value::as_str) == Some("response")
                && record.get("success").and_then(Value::as_bool) == Some(true)
        }) {
            self.runtime_failed = false;
        }
        self.append_stack_and_variables_invalidation(&mut records);
        records
    }

    fn append_stack_and_variables_invalidation(&mut self, records: &mut Vec<Value>) {
        let succeeded = records.first().is_some_and(|record| {
            record.get("type").and_then(Value::as_str) == Some("response")
                && record.get("success").and_then(Value::as_bool) == Some(true)
                && record.pointer("/body/terminated").and_then(Value::as_bool) != Some(true)
        });
        if succeeded && self.supports_invalidated_event {
            records.push(self.event("invalidated", json!({"areas":["stacks","variables"]})));
        }
    }
}
