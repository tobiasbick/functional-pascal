//! Runtime-failure stop-filter configuration.

use serde_json::{Map, Value, json};

use super::{DebugEngine, DebugStatus};
use crate::breakpoints::RuntimeFailurePolicy;
use crate::jsonl::encode::{invalid_state, missing_argument};
use crate::jsonl::protocol::{failure, success};

impl DebugEngine {
    pub(super) fn replace_runtime_failure_filters(
        &mut self,
        request_id: u64,
        command: &str,
        arguments: &Map<String, Value>,
    ) -> Vec<Value> {
        if !matches!(self.status, DebugStatus::Initialized | DebugStatus::Stopped) {
            return vec![invalid_state(request_id, command, self.status)];
        }
        let Some(requested) = arguments.get("filters").and_then(Value::as_array) else {
            return vec![missing_argument(request_id, command, "filters")];
        };
        let mut filters = Vec::with_capacity(requested.len());
        for (index, filter) in requested.iter().enumerate() {
            let Some(filter) = filter.as_str() else {
                return vec![failure(
                    request_id,
                    command,
                    "invalid_request",
                    format!("Runtime failure filter at index {index} must be a string."),
                    "Use `all` or exact advertised codes such as `F4001`.",
                )];
            };
            filters.push(filter.to_string());
        }
        let policy = match RuntimeFailurePolicy::parse(&filters) {
            Ok(policy) => policy,
            Err(error) => {
                return vec![failure(
                    request_id,
                    command,
                    "invalid_request",
                    error.message,
                    error.hint,
                )];
            }
        };
        self.runtime_failure_policy = policy;
        vec![success(request_id, command, json!({"filters": filters}))]
    }
}
