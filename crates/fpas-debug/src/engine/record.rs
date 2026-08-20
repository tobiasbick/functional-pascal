//! Typed debugger result envelopes shared by protocol adapters.

use serde_json::{Value, json};

use super::command::DebugCommand;

/// A debugger result emitted after one typed engine request.
#[derive(Debug, Clone)]
pub(crate) enum DebugRecord {
    /// Completion of one request.
    Response {
        /// Adapter correlation identifier.
        request_id: u64,
        /// Completed debugger operation.
        command: DebugCommand,
        /// Whether the operation succeeded.
        success: bool,
        /// Versioned success or failure payload.
        payload: Value,
    },
    /// An asynchronous debugger event.
    Event {
        /// Stable event name for adapter-specific translation.
        name: String,
        /// Versioned event payload.
        payload: Value,
    },
    /// A record that cannot yet be represented by the typed envelope.
    Unrecognized(Value),
}

impl DebugRecord {
    /// Decode an internal record while preserving every wire field.
    #[must_use]
    pub(crate) fn from_jsonl(value: Value) -> Self {
        match value.get("type").and_then(Value::as_str) {
            Some("response") => {
                let Some(request_id) = value.get("request_id").and_then(Value::as_u64) else {
                    return Self::Unrecognized(value);
                };
                let Some(command) = value.get("command").and_then(Value::as_str) else {
                    return Self::Unrecognized(value);
                };
                let Some(success) = value.get("success").and_then(Value::as_bool) else {
                    return Self::Unrecognized(value);
                };
                let payload = if success {
                    value.get("body").cloned().unwrap_or_else(|| json!({}))
                } else {
                    value.get("error").cloned().unwrap_or_else(|| json!({}))
                };
                Self::Response {
                    request_id,
                    command: DebugCommand::from_name(command),
                    success,
                    payload,
                }
            }
            Some("event") => {
                let Some(name) = value.get("event").and_then(Value::as_str) else {
                    return Self::Unrecognized(value);
                };
                Self::Event {
                    name: name.to_owned(),
                    payload: value.get("body").cloned().unwrap_or_else(|| json!({})),
                }
            }
            _ => Self::Unrecognized(value),
        }
    }

    /// Encode this result for the JSONL adapter.
    #[must_use]
    pub(crate) fn into_jsonl(self) -> Value {
        match self {
            Self::Response {
                request_id,
                command,
                success: true,
                payload,
            } => json!({
                "type":"response", "request_id":request_id, "command":command.name(),
                "success":true, "body":payload
            }),
            Self::Response {
                request_id,
                command,
                success: false,
                payload,
            } => json!({
                "type":"response", "request_id":request_id, "command":command.name(),
                "success":false, "error":payload
            }),
            Self::Event { name, payload } => json!({"type":"event", "event":name, "body":payload}),
            Self::Unrecognized(value) => value,
        }
    }
}
