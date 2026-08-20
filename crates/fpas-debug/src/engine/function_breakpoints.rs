//! Atomic logical function-breakpoint configuration.

use serde_json::{Map, Value, json};

use super::{DebugEngine, DebugStatus};
use crate::breakpoints::BreakpointPolicy;
use crate::jsonl::encode::{function_breakpoint_body, invalid_state, missing_argument};
use crate::jsonl::protocol::{event, failure, session_error, success};

impl DebugEngine {
    pub(super) fn replace_function_breakpoints(
        &mut self,
        request_id: u64,
        command: &str,
        arguments: &Map<String, Value>,
    ) -> Vec<Value> {
        if !matches!(self.status, DebugStatus::Initialized | DebugStatus::Stopped) {
            return vec![invalid_state(request_id, command, self.status)];
        }
        let Some(requested) = arguments.get("breakpoints").and_then(Value::as_array) else {
            return vec![missing_argument(request_id, command, "breakpoints")];
        };
        let limits = fpas_vm::DebugBreakpointLimits::default();
        let mut function_breakpoints = Vec::with_capacity(requested.len());
        let mut policies = Vec::with_capacity(requested.len());
        for (index, item) in requested.iter().enumerate() {
            let Some(item) = item.as_object() else {
                return vec![invalid_function_request(
                    request_id,
                    command,
                    index,
                    "expected an object",
                )];
            };
            if item
                .keys()
                .any(|field| !matches!(field.as_str(), "name" | "condition" | "hit_condition"))
            {
                return vec![invalid_function_request(
                    request_id,
                    command,
                    index,
                    "contains an unsupported field",
                )];
            }
            let Some(name) = item.get("name").and_then(Value::as_str) else {
                return vec![invalid_function_request(
                    request_id,
                    command,
                    index,
                    "requires string field `name`",
                )];
            };
            if name.is_empty() || name.len() > limits.max_function_name_bytes {
                return vec![invalid_function_request(
                    request_id,
                    command,
                    index,
                    &format!(
                        "name must contain 1..={} UTF-8 bytes",
                        limits.max_function_name_bytes
                    ),
                )];
            }
            let policy = match BreakpointPolicy::parse(
                item.get("condition").and_then(Value::as_str),
                item.get("hit_condition").and_then(Value::as_str),
                None,
                None,
            ) {
                Ok(policy) => policy,
                Err(error) => {
                    return vec![invalid_function_request(
                        request_id,
                        command,
                        index,
                        &format!("{} Help: {}", error.message, error.hint),
                    )];
                }
            };
            function_breakpoints.push(fpas_vm::FunctionBreakpoint {
                name: name.to_string(),
            });
            policies.push(policy);
        }
        let bound = match self
            .actor
            .session_mut()
            .map(|session| session.replace_function_breakpoints(function_breakpoints))
        {
            Some(Ok(bound)) => bound,
            Some(Err(error)) => return vec![session_error(request_id, command, error)],
            None => return vec![invalid_state(request_id, command, self.status)],
        };
        for id in self.function_breakpoint_ids.drain(..) {
            self.breakpoint_policies.remove(&id);
        }
        self.function_breakpoint_ids = bound.iter().map(|breakpoint| breakpoint.id).collect();
        for (breakpoint, policy) in bound.iter().zip(policies) {
            self.breakpoint_policies.insert(breakpoint.id, policy);
        }
        let bodies = bound
            .iter()
            .map(function_breakpoint_body)
            .collect::<Vec<_>>();
        let mut records = vec![success(request_id, command, json!({"breakpoints": bodies}))];
        records.extend(
            bound
                .iter()
                .map(function_breakpoint_body)
                .map(|body| event("breakpoint", body)),
        );
        records
    }
}

fn invalid_function_request(request_id: u64, command: &str, index: usize, detail: &str) -> Value {
    failure(
        request_id,
        command,
        "invalid_request",
        format!("Function breakpoint at index {index} {detail}."),
        "Send a bounded array of names with optional condition and hit_condition strings.",
    )
}
