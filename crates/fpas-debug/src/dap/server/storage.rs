//! DAP custom-request mapping for seeded empty-storage initialization.

use serde_json::{Map, Value, json};

use super::DapServer;

impl DapServer {
    pub(super) fn initialize_storage(
        &mut self,
        request_seq: u64,
        command: &str,
        arguments: &Value,
    ) -> Vec<Value> {
        let mut records = self.core_request(
            request_seq,
            command,
            "storage.initialize",
            jsonl_arguments(arguments),
        );
        self.append_variables_invalidation(&mut records);
        records
    }
}

/// Translate one empty-storage custom-request result into DAP naming.
pub(super) fn response_body(command: &str, body: &Value) -> Option<Value> {
    if command != "fpas/initializeStorage" {
        return None;
    }
    Some(json!({
        "root": body.get("root"),
        "target": body.get("target"),
        "rootValue": body.get("root_value"),
        "value": body.get("value"),
        "type": body.get("type"),
        "variablesReference": body.get("variables_reference"),
        "namedVariables": body.get("named_variables"),
        "indexedVariables": body.get("indexed_variables")
    }))
}

fn jsonl_arguments(arguments: &Value) -> Value {
    let mut mapped = Map::from_iter([
        (
            "frame_id".to_string(),
            arguments.get("frameId").cloned().unwrap_or(Value::Null),
        ),
        (
            "target".to_string(),
            arguments.get("target").cloned().unwrap_or(Value::Null),
        ),
        (
            "initializer".to_string(),
            arguments.get("initializer").cloned().unwrap_or(Value::Null),
        ),
        (
            "expression".to_string(),
            arguments.get("expression").cloned().unwrap_or(Value::Null),
        ),
    ]);
    if let Some(object) = arguments.as_object() {
        for (key, value) in object {
            if !matches!(
                key.as_str(),
                "frameId" | "target" | "initializer" | "expression"
            ) {
                mapped.insert(key.clone(), value.clone());
            }
        }
    }
    Value::Object(mapped)
}
