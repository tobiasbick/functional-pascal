//! DAP custom-request mapping for seeded empty-storage initialization.

use serde_json::{Value, json};

use super::DapServer;
use super::args;
use crate::engine::{DebugOp, ResponseBody};

impl DapServer {
    pub(super) fn initialize_storage(
        &mut self,
        request_seq: u64,
        command: &str,
        arguments: &Value,
    ) -> Vec<Value> {
        if let Err(message) = args::reject_unknown_fields(
            arguments,
            &["frameId", "target", "initializer", "expression"],
        ) {
            return vec![self.structured_failure(
                request_seq,
                command,
                "invalid_request",
                &message,
                "Pass only `frameId`, `target`, `initializer`, and `expression`.",
            )];
        }
        self.mutating_request(
            request_seq,
            command,
            (|| {
                Ok(DebugOp::StorageInitialize {
                    frame_id: args::optional_u64(arguments, "frameId")?,
                    target: args::required_string(arguments, "target")?,
                    initializer: args::required_string(arguments, "initializer")?,
                    expression: args::required_string(arguments, "expression")?,
                })
            })(),
        )
    }
}

/// Translate one empty-storage custom-request result into DAP naming.
pub(super) fn response_body(command: &str, body: &ResponseBody) -> Option<Value> {
    if command != "fpas/initializeStorage" {
        return None;
    }
    let ResponseBody::Storage(result) = body else {
        return None;
    };
    Some(json!({
        "root": result.root,
        "target": result.target,
        "rootValue": result.root_value,
        "value": result.value.value,
        "type": result.value.type_name,
        "variablesReference": result.value.variables_reference,
        "namedVariables": result.value.named_variables,
        "indexedVariables": result.value.indexed_variables
    }))
}
