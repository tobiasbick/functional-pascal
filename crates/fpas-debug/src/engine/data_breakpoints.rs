//! Atomic JSONL mapping for global data breakpoints.

use serde_json::{Map, Value, json};

use super::breakpoints::parse_assign_argument;
use super::location::parse_identity;
use super::{DebugEngine, DebugStatus};
use crate::jsonl::encode::{data_breakpoint_body, invalid_state, missing_argument};
use crate::jsonl::protocol::{event, failure, session_error, success};

impl DebugEngine {
    pub(super) fn replace_data_breakpoints(
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
        let mut data_breakpoints = Vec::with_capacity(requested.len());
        let mut assigns = Vec::with_capacity(requested.len());
        for (index, item) in requested.iter().enumerate() {
            let Some(item) = item.as_object() else {
                return vec![invalid_data_request(
                    request_id,
                    command,
                    index,
                    "expected an object",
                )];
            };
            if item
                .keys()
                .any(|field| !matches!(field.as_str(), "identity" | "access" | "assign"))
            {
                return vec![invalid_data_request(
                    request_id,
                    command,
                    index,
                    "contains an unsupported field",
                )];
            }
            let Some(identity) = item.get("identity").and_then(parse_identity) else {
                return vec![invalid_data_request(
                    request_id,
                    command,
                    index,
                    "requires a location identity from `location.describe`",
                )];
            };
            let access = match item.get("access").and_then(Value::as_str) {
                None => fpas_vm::DataBreakpointAccess::Write,
                Some(value) => match fpas_vm::DataBreakpointAccess::parse(value) {
                    Some(access) => access,
                    None => {
                        return vec![invalid_data_request(
                            request_id,
                            command,
                            index,
                            "access must be write, change, or read",
                        )];
                    }
                },
            };
            data_breakpoints.push(fpas_vm::DataBreakpoint { identity, access });
            let assign = match parse_assign_argument(item.get("assign")) {
                Ok(assign) => assign,
                Err(message) => {
                    return vec![failure(
                        request_id,
                        command,
                        "invalid_request",
                        format!("Data breakpoint at index {index}: {message}"),
                        "Send `assign.identity` from `location.describe` and one replacement `expression`.",
                    )];
                }
            };
            assigns.push(assign);
        }
        let bound = match self
            .actor
            .session_mut()
            .map(|session| session.replace_data_breakpoints(data_breakpoints))
        {
            Some(Ok(bound)) => bound,
            Some(Err(error)) => return vec![session_error(request_id, command, error)],
            None => return vec![invalid_state(request_id, command, self.status)],
        };
        for id in self.data_breakpoint_ids.drain(..) {
            self.breakpoint_policies.remove(&id);
        }
        self.data_breakpoint_ids = bound.iter().map(|breakpoint| breakpoint.id).collect();
        for (breakpoint, assign) in bound.iter().zip(assigns) {
            if let Some(assign) = assign {
                self.breakpoint_policies.insert(
                    breakpoint.id,
                    crate::breakpoints::BreakpointPolicy::with_assign(assign),
                );
            }
        }
        let bodies = bound.iter().map(data_breakpoint_body).collect::<Vec<_>>();
        let mut records = vec![success(request_id, command, json!({"breakpoints": bodies}))];
        records.extend(
            bound
                .iter()
                .map(data_breakpoint_body)
                .map(|body| event("breakpoint", body)),
        );
        records
    }
}

fn invalid_data_request(request_id: u64, command: &str, index: usize, detail: &str) -> Value {
    failure(
        request_id,
        command,
        "invalid_request",
        format!("Data breakpoint at index {index} {detail}."),
        "Send identities from `location.describe` with access write or change.",
    )
}
