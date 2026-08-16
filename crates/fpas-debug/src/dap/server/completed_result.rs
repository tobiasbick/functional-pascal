//! DAP custom-request mapping for retained completed task results.

use serde_json::{Value, json};

use super::DapServer;

/// Translate one retained-result replacement into DAP naming.
pub(super) fn response_body(command: &str, body: &Value) -> Option<Value> {
    (command == "fpas/replaceTaskResult").then(|| {
        json!({
            "taskId": body.get("task_id"),
            "value": body.get("result"),
            "type": body.get("type_name"),
            "variablesReference": body.get("variables_reference"),
            "namedVariables": body.get("named_variables"),
            "indexedVariables": body.get("indexed_variables")
        })
    })
}

impl DapServer {
    pub(super) fn replace_completed_task_result(
        &mut self,
        request_seq: u64,
        command: &str,
        arguments: &Value,
    ) -> Vec<Value> {
        let mut records = self.core_request(
            request_seq,
            command,
            "task.result.replace",
            json!({
                "task_id": arguments.get("taskId").cloned().unwrap_or(Value::Null),
                "frame_id": arguments.get("frameId").cloned().unwrap_or(Value::Null),
                "expression": arguments.get("expression").cloned().unwrap_or(Value::Null)
            }),
        );
        if replacement_succeeded(&records) && self.supports_invalidated_event {
            records.push(self.event("invalidated", json!({"areas":["variables"]})));
        }
        records
    }
}

fn replacement_succeeded(records: &[Value]) -> bool {
    records.first().is_some_and(|record| {
        record.get("type").and_then(Value::as_str) == Some("response")
            && record.get("success").and_then(Value::as_bool) == Some(true)
    })
}
