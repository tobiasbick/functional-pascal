//! DAP custom-request mapping for forced return.

use serde_json::{Value, json};

use super::DapServer;
use super::args;
use super::values;
use crate::engine::{DebugOp, ResponseBody};

/// Translate one forced-return result into DAP naming.
pub(super) fn response_body(command: &str, body: &ResponseBody) -> Option<Value> {
    if command != "fpas/forceReturn" {
        return None;
    }
    let ResponseBody::ForcedReturn(result) = body else {
        return None;
    };
    Some(json!({
        "value": result.value,
        "type": result.type_name,
        "variablesReference": result.variables_reference,
        "namedVariables": result.named_variables,
        "indexedVariables": result.indexed_variables,
        "unwoundFrames": result.unwound_frames,
        "taskId": result.task_id,
        "frame": result.frame.as_ref().map(values::frame_json),
        "terminated": result.terminated
    }))
}

impl DapServer {
    pub(super) fn force_return(
        &mut self,
        request_seq: u64,
        command: &str,
        arguments: &Value,
    ) -> Vec<Value> {
        let op: Result<DebugOp, String> = (|| {
            Ok(DebugOp::FrameReturn {
                frame_id: args::required_u64(arguments, "frameId")?,
                expression: args::optional_expression(arguments, "expression")?,
            })
        })();
        let mut records = match op {
            Ok(op) => self.core_request(request_seq, command, op),
            Err(message) => return vec![self.failure(request_seq, command, &message)],
        };
        if response_succeeded(&records) {
            self.runtime_failed = false;
        }
        self.append_stack_and_variables_invalidation(&mut records);
        records
    }

    fn append_stack_and_variables_invalidation(&mut self, records: &mut Vec<Value>) {
        let succeeded = records.first().is_some_and(|record| {
            record.get("type").and_then(Value::as_str) == Some("response")
                && record.get("success").and_then(Value::as_bool) == Some(true)
                && record.pointer("/body/terminated").and_then(Value::as_bool) != Some(true)
        });
        if succeeded && self.supports_invalidated_event {
            records.push(self.event("invalidated", json!({"areas":["stacks","variables"]})));
        }
    }
}

fn response_succeeded(records: &[Value]) -> bool {
    records.first().is_some_and(|record| {
        record.get("type").and_then(Value::as_str) == Some("response")
            && record.get("success").and_then(Value::as_bool) == Some(true)
    })
}
