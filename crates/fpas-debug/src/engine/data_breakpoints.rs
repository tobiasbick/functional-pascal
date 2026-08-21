//! Atomic global data breakpoints.

use super::breakpoints::assign_from_op;
use super::record::{DebugEvent, DebugRecord, ResponseBody};
use super::reply::{event, invalid_request, invalid_state, ok, session_error};
use super::request::DataBreakpointOp;
use super::{DebugEngine, DebugStatus};

impl DebugEngine {
    pub(super) fn replace_data_breakpoints(
        &mut self,
        request_id: u64,
        command: &str,
        requested: Vec<DataBreakpointOp>,
    ) -> Vec<DebugRecord> {
        if !matches!(self.status, DebugStatus::Initialized | DebugStatus::Stopped) {
            return vec![invalid_state(request_id, command, self.status)];
        }
        let mut data_breakpoints = Vec::with_capacity(requested.len());
        let mut assigns = Vec::with_capacity(requested.len());
        for (index, item) in requested.into_iter().enumerate() {
            data_breakpoints.push(fpas_vm::DataBreakpoint {
                identity: item.identity,
                access: item.access,
            });
            let assign = match item.assign.map(assign_from_op).transpose() {
                Ok(assign) => assign,
                Err(message) => {
                    return vec![invalid_request(
                        request_id,
                        command,
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
        let mut records = vec![ok(
            request_id,
            command,
            ResponseBody::DataBreakpoints {
                breakpoints: bound.clone(),
            },
        )];
        records.extend(
            bound
                .into_iter()
                .map(|breakpoint| event(DebugEvent::DataBreakpoint(breakpoint))),
        );
        records
    }
}
