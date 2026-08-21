//! DAP custom-request mapping for retained completed task results.

use serde_json::{Value, json};

use super::DapServer;
use super::args;
use crate::engine::{DebugOp, ResponseBody};

/// Translate one retained-result replacement into DAP naming.
pub(super) fn response_body(command: &str, body: &ResponseBody) -> Option<Value> {
    if command != "fpas/replaceTaskResult" {
        return None;
    }
    let ResponseBody::TaskResult(result) = body else {
        return None;
    };
    Some(json!({
        "taskId": result.task_id,
        "value": result.value,
        "type": result.type_name,
        "variablesReference": result.variables_reference,
        "namedVariables": result.named_variables,
        "indexedVariables": result.indexed_variables
    }))
}

impl DapServer {
    pub(super) fn replace_completed_task_result(
        &mut self,
        request_seq: u64,
        command: &str,
        arguments: &Value,
    ) -> Vec<Value> {
        let op: Result<DebugOp, String> = (|| {
            Ok(DebugOp::TaskResultReplace {
                task_id: args::required_u64(arguments, "taskId")?,
                expression: args::optional_expression(arguments, "expression")?,
                frame_id: args::optional_u64(arguments, "frameId")?,
            })
        })();
        match op {
            Ok(op) => {
                let mut records = self.core_request(request_seq, command, op);
                if replacement_succeeded(&records) && self.supports_invalidated_event {
                    records.push(self.event("invalidated", json!({"areas":["variables"]})));
                }
                records
            }
            Err(message) => vec![self.failure(request_seq, command, &message)],
        }
    }
}

fn replacement_succeeded(records: &[Value]) -> bool {
    records.first().is_some_and(|record| {
        record.get("type").and_then(Value::as_str) == Some("response")
            && record.get("success").and_then(Value::as_bool) == Some(true)
    })
}
