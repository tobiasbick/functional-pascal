//! JSON construction for the JSONL adapter envelope.

use serde_json::{Value, json};

pub(crate) fn success(request_id: u64, command: &str, body: Value) -> Value {
    json!({
        "type": "response",
        "request_id": request_id,
        "command": command,
        "success": true,
        "body": body,
    })
}

pub(crate) fn failure(
    request_id: u64,
    command: &str,
    code: &str,
    message: impl Into<String>,
    help: impl Into<String>,
) -> Value {
    json!({
        "type": "response",
        "request_id": request_id,
        "command": command,
        "success": false,
        "error": {
            "code": code,
            "message": message.into(),
            "help": help.into(),
        },
    })
}

pub(crate) fn event(name: &str, body: Value) -> Value {
    json!({"type": "event", "event": name, "body": body})
}
