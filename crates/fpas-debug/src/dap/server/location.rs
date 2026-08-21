//! DAP custom-request mapping for durable data-location identities.

use serde_json::Value;

use super::DapServer;
use super::args;
use super::values;
use crate::engine::{DebugOp, ResponseBody};

impl DapServer {
    pub(super) fn describe_location(
        &mut self,
        request_seq: u64,
        command: &str,
        arguments: &Value,
    ) -> Vec<Value> {
        match (
            args::required_u64(arguments, "variablesReference"),
            args::required_string(arguments, "name"),
        ) {
            (Ok(variables_reference), Ok(name)) => self.core_request(
                request_seq,
                command,
                DebugOp::LocationDescribe {
                    variables_reference,
                    name,
                },
            ),
            (Err(message), _) | (_, Err(message)) => {
                vec![self.failure(request_seq, command, &message)]
            }
        }
    }
}

/// Translate one location custom-request result into DAP naming.
pub(super) fn response_body(command: &str, body: &ResponseBody) -> Option<Value> {
    if command != "fpas/locationDescribe" {
        return None;
    }
    let ResponseBody::Location(location) = body else {
        return None;
    };
    let mut result = serde_json::Map::from_iter([
        ("kind".into(), Value::String(location.kind.as_str().into())),
        (
            "lifetime".into(),
            Value::String(location.lifetime.as_str().into()),
        ),
        ("descendant".into(), Value::Bool(location.descendant)),
    ]);
    if let Some(identity) = location.identity {
        result.insert("identity".into(), values::identity_json(identity));
    }
    Some(Value::Object(result))
}
