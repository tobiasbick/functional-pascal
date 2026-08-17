//! JSONL mapping for durable data-location identities.

use serde_json::{Map, Value, json};

use super::{JsonlServer, ServerStatus};
use crate::jsonl::encode::{invalid_state, missing_argument};
use crate::jsonl::protocol::{session_error, success};

impl JsonlServer {
    pub(super) fn describe_location(
        &mut self,
        request_id: u64,
        command: &str,
        arguments: &Map<String, Value>,
    ) -> Vec<Value> {
        if self.status != ServerStatus::Stopped {
            return vec![invalid_state(request_id, command, self.status)];
        }
        let Some(variables_reference) =
            arguments.get("variables_reference").and_then(Value::as_u64)
        else {
            return vec![missing_argument(request_id, command, "variables_reference")];
        };
        let Some(name) = arguments.get("name").and_then(Value::as_str) else {
            return vec![missing_argument(request_id, command, "name")];
        };
        let Some(session) = self.actor.session() else {
            return vec![invalid_state(request_id, command, self.status)];
        };
        match session.describe_data_location(variables_reference, name) {
            Ok(location) => vec![success(request_id, command, location_body(&location))],
            Err(error) => vec![session_error(request_id, command, error)],
        }
    }
}

fn location_body(location: &fpas_vm::DebugDataLocation) -> Value {
    let mut body = json!({
        "kind": location.kind.as_str(),
        "lifetime": location.lifetime.as_str(),
        "descendant": location.descendant,
    });
    if let (Value::Object(body), Some(identity)) = (&mut body, location.identity) {
        body.insert(
            "identity".into(),
            crate::jsonl::encode::identity_body(identity),
        );
    }
    body
}

pub(super) fn parse_identity(value: &Value) -> Option<fpas_vm::DebugDataLocationIdentity> {
    let object = value.as_object()?;
    if let Some(index) = object.get("index").and_then(Value::as_u64) {
        return Some(fpas_vm::DebugDataLocationIdentity::Global { index });
    }
    Some(fpas_vm::DebugDataLocationIdentity::FrameRegister {
        task_id: object.get("task_id").and_then(Value::as_u64)?,
        function: object.get("function").and_then(Value::as_u64)?,
        register: object.get("register").and_then(Value::as_u64)?,
    })
}
