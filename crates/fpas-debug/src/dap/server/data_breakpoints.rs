//! DAP data-breakpoint translation onto JSONL location identities.

use serde_json::{Value, json};

use super::DapServer;

impl DapServer {
    pub(super) fn data_breakpoint_info(
        &mut self,
        request_seq: u64,
        arguments: &Value,
    ) -> Vec<Value> {
        self.core_request(
            request_seq,
            "dataBreakpointInfo",
            "location.describe",
            json!({
                "variables_reference": arguments.get("variablesReference").cloned().unwrap_or(Value::Null),
                "name": arguments.get("name").cloned().unwrap_or(Value::Null)
            }),
        )
    }

    pub(super) fn set_data_breakpoints(
        &mut self,
        request_seq: u64,
        arguments: &Value,
    ) -> Vec<Value> {
        let requested = arguments
            .get("breakpoints")
            .and_then(Value::as_array)
            .cloned()
            .unwrap_or_default();
        let mut breakpoints = Vec::with_capacity(requested.len());
        for breakpoint in requested {
            let Some(data_id) = breakpoint.get("dataId").and_then(Value::as_str) else {
                return vec![self.failure(
                    request_seq,
                    "setDataBreakpoints",
                    "setDataBreakpoints requires dataId from dataBreakpointInfo.",
                )];
            };
            let Some(identity) = identity_from_data_id(data_id) else {
                return vec![self.failure(
                    request_seq,
                    "setDataBreakpoints",
                    "setDataBreakpoints dataId must name a global from dataBreakpointInfo.",
                )];
            };
            let access = match breakpoint.get("accessType").and_then(Value::as_str) {
                None | Some("write") => "write",
                Some("change") => "change",
                Some("read" | "readWrite") => "read",
                Some(other) => {
                    return vec![self.failure(
                        request_seq,
                        "setDataBreakpoints",
                        &format!("Unsupported data-breakpoint accessType `{other}`."),
                    )];
                }
            };
            breakpoints.push(json!({
                "identity": identity,
                "access": access
            }));
        }
        self.core_request(
            request_seq,
            "setDataBreakpoints",
            "data_breakpoints.replace",
            json!({"breakpoints": breakpoints}),
        )
    }
}

pub(super) fn response_body(command: &str, body: &Value) -> Option<Value> {
    match command {
        "dataBreakpointInfo" => Some(info_body(body)),
        "setDataBreakpoints" => Some(json!({
            "breakpoints": body
                .get("breakpoints")
                .and_then(Value::as_array)
                .into_iter()
                .flatten()
                .map(|breakpoint| json!({
                    "id": breakpoint.get("breakpoint_id"),
                    "verified": breakpoint.get("verified"),
                    "message": breakpoint.get("message")
                }))
                .collect::<Vec<_>>()
        })),
        _ => None,
    }
}

fn info_body(body: &Value) -> Value {
    match parse_global_index(body) {
        Some(index) => json!({
            "dataId": format!("g:{index}"),
            "description": format!("global slot {index}"),
            "accessTypes": ["write"],
            "canPersist": false
        }),
        None => json!({
            "dataId": Value::Null,
            "description": "Only executable globals are watchable; frame registers and capture cells are not.",
            "accessTypes": ["write"],
            "canPersist": false
        }),
    }
}

fn parse_global_index(body: &Value) -> Option<u64> {
    body.pointer("/identity/index").and_then(Value::as_u64)
}

fn identity_from_data_id(data_id: &str) -> Option<Value> {
    let index = data_id.strip_prefix("g:")?.parse::<u64>().ok()?;
    Some(json!({"index": index}))
}
