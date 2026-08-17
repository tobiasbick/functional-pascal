//! DAP custom-request mapping for durable data-location identities.

use serde_json::{Value, json};

use super::DapServer;

impl DapServer {
    pub(super) fn describe_location(
        &mut self,
        request_seq: u64,
        command: &str,
        arguments: &Value,
    ) -> Vec<Value> {
        self.core_request(
            request_seq,
            command,
            "location.describe",
            json!({
                "variables_reference": arguments.get("variablesReference").cloned().unwrap_or(Value::Null),
                "name": arguments.get("name").cloned().unwrap_or(Value::Null)
            }),
        )
    }
}

/// Translate one location custom-request result into DAP naming.
pub(super) fn response_body(command: &str, body: &Value) -> Option<Value> {
    if command != "fpas/locationDescribe" {
        return None;
    }
    let mut result = serde_json::Map::from_iter([
        (
            "kind".to_string(),
            body.get("kind").cloned().unwrap_or(Value::Null),
        ),
        (
            "lifetime".to_string(),
            body.get("lifetime").cloned().unwrap_or(Value::Null),
        ),
        (
            "descendant".to_string(),
            body.get("descendant").cloned().unwrap_or(Value::Null),
        ),
    ]);
    if let Some(Value::Object(identity)) = body.get("identity") {
        let mut mapped = serde_json::Map::new();
        if let Some(index) = identity.get("index") {
            mapped.insert("index".to_string(), index.clone());
        }
        if let Some(task_id) = identity.get("task_id") {
            mapped.insert("taskId".to_string(), task_id.clone());
        }
        if let Some(function) = identity.get("function") {
            mapped.insert("function".to_string(), function.clone());
        }
        if let Some(register) = identity.get("register") {
            mapped.insert("register".to_string(), register.clone());
        }
        result.insert("identity".to_string(), Value::Object(mapped));
    }
    Some(Value::Object(result))
}
