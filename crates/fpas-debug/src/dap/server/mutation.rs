//! DAP `setVariable` mapping and client-negotiated variable invalidation.

use serde_json::{Value, json};

use super::DapServer;
use super::args;
use super::values;
use crate::engine::{DebugOp, ResponseBody};

/// Translate one custom mutation result into DAP naming without null-only metadata.
pub(super) fn custom_response_body(command: &str, body: &ResponseBody) -> Option<Value> {
    if !matches!(
        command,
        "fpas/dictionaryInsert"
            | "fpas/dictionaryRemove"
            | "fpas/dictionaryReplaceKey"
            | "fpas/arrayInsert"
            | "fpas/arrayRemove"
            | "fpas/stringReplaceCharacter"
    ) {
        return None;
    }
    let result = match body {
        ResponseBody::Dictionary(mutation) => {
            let mut result = values::variable_value_json(&mutation.dictionary);
            if let Some(removed) = &mutation.removed {
                result.insert("removed".into(), Value::String(removed.clone()));
            }
            if let Some(old_key) = &mutation.old_key {
                result.insert("oldKey".into(), Value::String(old_key.clone()));
            }
            if let Some(new_key) = &mutation.new_key {
                result.insert("newKey".into(), Value::String(new_key.clone()));
            }
            result
        }
        ResponseBody::Array(mutation) => {
            let mut result = values::variable_value_json(&mutation.array);
            result.insert("index".into(), Value::from(mutation.index));
            if let Some(removed) = &mutation.removed {
                result.insert("removed".into(), Value::String(removed.clone()));
            }
            result
        }
        ResponseBody::StringCharacter(mutation) => {
            let mut result = values::variable_value_json(&mutation.string);
            result.insert("index".into(), Value::from(mutation.index));
            result.insert(
                "oldCharacter".into(),
                Value::String(mutation.old_character.clone()),
            );
            result.insert(
                "newCharacter".into(),
                Value::String(mutation.new_character.clone()),
            );
            result
        }
        _ => return None,
    };
    Some(Value::Object(result))
}

impl DapServer {
    pub(super) fn set_variable(
        &mut self,
        request_seq: u64,
        command: &str,
        arguments: &Value,
    ) -> Vec<Value> {
        if arguments
            .get("format")
            .is_some_and(|format| !format.is_null() && format != &json!({}))
        {
            return vec![self.failure(
                request_seq,
                command,
                "Non-default DAP value formatting is not supported for setVariable.",
            )];
        }
        let op: Result<DebugOp, String> = (|| {
            Ok(DebugOp::VariableSet {
                variables_reference: args::required_u64(arguments, "variablesReference")?,
                name: args::required_string(arguments, "name")?,
                expression: args::required_string(arguments, "value")?,
            })
        })();
        self.mutating_request(request_seq, command, op)
    }

    /// Maps standard DAP `setExpression` arguments onto the debug engine.
    pub(super) fn set_expression(
        &mut self,
        request_seq: u64,
        command: &str,
        arguments: &Value,
    ) -> Vec<Value> {
        if arguments
            .get("format")
            .is_some_and(|format| !format.is_null() && format != &json!({}))
        {
            return vec![self.failure(
                request_seq,
                command,
                "Non-default DAP value formatting is not supported for setExpression.",
            )];
        }
        let op: Result<DebugOp, String> = (|| {
            Ok(DebugOp::ExpressionSet {
                target: args::required_string(arguments, "expression")?,
                expression: args::required_string(arguments, "value")?,
                frame_id: args::optional_u64(arguments, "frameId")?,
            })
        })();
        self.mutating_request(request_seq, command, op)
    }

    pub(super) fn mutating_request(
        &mut self,
        request_seq: u64,
        command: &str,
        op: Result<DebugOp, String>,
    ) -> Vec<Value> {
        match op {
            Ok(op) => {
                let mut records = self.core_request(request_seq, command, op);
                self.append_variables_invalidation(&mut records);
                records
            }
            Err(message) => vec![self.failure(request_seq, command, &message)],
        }
    }

    pub(super) fn append_variables_invalidation(&mut self, records: &mut Vec<Value>) {
        let succeeded = records.first().is_some_and(|record| {
            record.get("type").and_then(Value::as_str) == Some("response")
                && record.get("success").and_then(Value::as_bool) == Some(true)
        });
        if succeeded && self.supports_invalidated_event {
            records.push(self.event("invalidated", json!({"areas":["variables"]})));
        }
    }
}
