//! Logical breakpoint configuration.

use super::record::{DebugEvent, DebugRecord, ResponseBody};
use super::reply::{event, invalid_request, invalid_state, ok, session_error};
use super::request::AssignOp;
use super::{DebugEngine, DebugStatus};
use crate::breakpoints::{BreakpointAssign, BreakpointPolicy};
use crate::evaluation::parse_debug_expression;

impl DebugEngine {
    #[allow(clippy::too_many_arguments)]
    pub(super) fn set_breakpoint(
        &mut self,
        request_id: u64,
        command: &str,
        source: String,
        line: u32,
        column: Option<u32>,
        assign: Option<AssignOp>,
        condition: Option<String>,
        hit_condition: Option<String>,
        log_message: Option<String>,
    ) -> Vec<DebugRecord> {
        if !matches!(self.status, DebugStatus::Initialized | DebugStatus::Stopped) {
            return vec![invalid_state(request_id, command, self.status)];
        }
        let assign = match assign.map(assign_from_op).transpose() {
            Ok(assign) => assign,
            Err(message) => {
                return vec![invalid_request(
                    request_id,
                    command,
                    message,
                    "Send `assign.identity` from `location.describe` and one replacement `expression`.",
                )];
            }
        };
        let policy = match BreakpointPolicy::parse(
            condition.as_deref(),
            hit_condition.as_deref(),
            log_message.as_deref(),
            assign,
        ) {
            Ok(policy) => policy,
            Err(error) => {
                return vec![ok(
                    request_id,
                    command,
                    ResponseBody::UnverifiedBreakpoint {
                        source,
                        line,
                        column,
                        message: format!("{} Help: {}", error.message, error.hint),
                        error_code: error.code.to_string(),
                        error_offset: error.offset,
                        error_length: error.length,
                    },
                )];
            }
        };
        let breakpoint = match self.actor.session_mut().map(|session| {
            session.set_breakpoint(fpas_vm::SourceBreakpoint {
                source,
                line,
                column,
            })
        }) {
            Some(Ok(breakpoint)) => breakpoint,
            Some(Err(error)) => return vec![session_error(request_id, command, error)],
            None => return vec![invalid_state(request_id, command, self.status)],
        };
        self.breakpoint_policies.insert(breakpoint.id, policy);
        vec![
            ok(
                request_id,
                command,
                ResponseBody::Breakpoint(breakpoint.clone()),
            ),
            event(DebugEvent::SourceBreakpoint(breakpoint)),
        ]
    }

    pub(super) fn clear_breakpoint(
        &mut self,
        request_id: u64,
        command: &str,
        id: u64,
    ) -> Vec<DebugRecord> {
        if !matches!(self.status, DebugStatus::Initialized | DebugStatus::Stopped) {
            return vec![invalid_state(request_id, command, self.status)];
        }
        self.breakpoint_policies.remove(&id);
        match self
            .actor
            .session_mut()
            .map(|session| session.clear_breakpoint(id))
        {
            Some(Ok(())) => vec![ok(
                request_id,
                command,
                ResponseBody::BreakpointCleared { breakpoint_id: id },
            )],
            Some(Err(error)) => vec![session_error(request_id, command, error)],
            None => vec![invalid_state(request_id, command, self.status)],
        }
    }
}

pub(super) fn assign_from_op(assign: AssignOp) -> Result<BreakpointAssign, String> {
    let expression = parse_debug_expression(
        &assign.expression,
        fpas_vm::DebugEvaluationLimits::default(),
    )
    .map_err(|error| format!("{} Help: {}", error.message, error.hint))?;
    BreakpointAssign::new(assign.identity, expression)
}
