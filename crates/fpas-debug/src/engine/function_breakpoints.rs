//! Atomic logical function-breakpoint configuration.

use super::record::{DebugEvent, DebugRecord, ResponseBody};
use super::reply::{event, invalid_request, invalid_state, ok, session_error};
use super::request::FunctionBreakpointOp;
use super::{DebugEngine, DebugStatus};
use crate::breakpoints::BreakpointPolicy;

impl DebugEngine {
    pub(super) fn replace_function_breakpoints(
        &mut self,
        request_id: u64,
        command: &str,
        requested: Vec<FunctionBreakpointOp>,
    ) -> Vec<DebugRecord> {
        if !matches!(self.status, DebugStatus::Initialized | DebugStatus::Stopped) {
            return vec![invalid_state(request_id, command, self.status)];
        }
        let limits = fpas_vm::DebugBreakpointLimits::default();
        let mut function_breakpoints = Vec::with_capacity(requested.len());
        let mut policies = Vec::with_capacity(requested.len());
        for (index, item) in requested.into_iter().enumerate() {
            if item.name.is_empty() || item.name.len() > limits.max_function_name_bytes {
                return vec![invalid_request(
                    request_id,
                    command,
                    format!(
                        "Function breakpoint at index {index} name must contain 1..={} UTF-8 bytes.",
                        limits.max_function_name_bytes
                    ),
                    "Send a bounded array of names with optional condition and hit_condition strings.",
                )];
            }
            let policy = match BreakpointPolicy::parse(
                item.condition.as_deref(),
                item.hit_condition.as_deref(),
                None,
                None,
            ) {
                Ok(policy) => policy,
                Err(error) => {
                    return vec![invalid_request(
                        request_id,
                        command,
                        format!(
                            "Function breakpoint at index {index} {} Help: {}.",
                            error.message, error.hint
                        ),
                        "Send a bounded array of names with optional condition and hit_condition strings.",
                    )];
                }
            };
            function_breakpoints.push(fpas_vm::FunctionBreakpoint { name: item.name });
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
        let mut records = vec![ok(
            request_id,
            command,
            ResponseBody::FunctionBreakpoints {
                breakpoints: bound.clone(),
            },
        )];
        records.extend(
            bound
                .into_iter()
                .map(|breakpoint| event(DebugEvent::FunctionBreakpoint(breakpoint))),
        );
        records
    }
}
