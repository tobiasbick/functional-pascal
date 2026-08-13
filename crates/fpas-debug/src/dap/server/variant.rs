//! DAP custom-request mapping for variant discovery and construction.

use serde_json::{Value, json};

use super::DapServer;

impl DapServer {
    pub(super) fn describe_variant(
        &mut self,
        request_seq: u64,
        command: &str,
        arguments: &Value,
    ) -> Vec<Value> {
        self.core_request(
            request_seq,
            command,
            "variant.describe",
            json!({
                "frame_id": arguments.get("frameId").cloned().unwrap_or(Value::Null),
                "target": arguments.get("target").cloned().unwrap_or(Value::Null)
            }),
        )
    }

    pub(super) fn construct_variant(
        &mut self,
        request_seq: u64,
        command: &str,
        arguments: &Value,
    ) -> Vec<Value> {
        let mut records = self.core_request(
            request_seq,
            command,
            "variant.construct",
            json!({
                "frame_id": arguments.get("frameId").cloned().unwrap_or(Value::Null),
                "target": arguments.get("target").cloned().unwrap_or(Value::Null),
                "variant": arguments.get("variant").cloned().unwrap_or(Value::Null),
                "fields": arguments.get("fields").cloned().unwrap_or(Value::Null)
            }),
        );
        self.append_variables_invalidation(&mut records);
        records
    }
}

/// Translate one variant custom-request result into DAP naming.
pub(super) fn response_body(command: &str, body: &Value) -> Option<Value> {
    match command {
        "fpas/variantDescribe" => Some(json!({
            "target": body.get("target"),
            "typeName": body.get("type_name"),
            "variants": body.get("variants").and_then(Value::as_array).into_iter().flatten().map(|variant| {
                json!({
                    "name": variant.get("name"),
                    "fields": variant.get("fields").and_then(Value::as_array).into_iter().flatten().map(|field| {
                        json!({
                            "name": field.get("name"),
                            "typeName": field.get("type_name")
                        })
                    }).collect::<Vec<_>>()
                })
            }).collect::<Vec<_>>()
        })),
        "fpas/variantConstruct" => {
            let mut result = serde_json::Map::from_iter([
                (
                    "value".to_string(),
                    body.get("result").cloned().unwrap_or(Value::Null),
                ),
                (
                    "type".to_string(),
                    body.get("type_name").cloned().unwrap_or(Value::Null),
                ),
                (
                    "variablesReference".to_string(),
                    body.get("variables_reference")
                        .cloned()
                        .unwrap_or(Value::Null),
                ),
                (
                    "namedVariables".to_string(),
                    body.get("named_variables").cloned().unwrap_or(Value::Null),
                ),
                (
                    "indexedVariables".to_string(),
                    body.get("indexed_variables")
                        .cloned()
                        .unwrap_or(Value::Null),
                ),
            ]);
            if let Some(variant) = body.get("variant") {
                result.insert("variant".to_string(), variant.clone());
            }
            Some(Value::Object(result))
        }
        _ => None,
    }
}
