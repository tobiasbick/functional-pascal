//! Read-only stopped-frame evaluation request handling.

use super::command::DebugCommand;
use super::record::{DebugRecord, ResponseBody};
use super::reply::{invalid_state, ok, parse_error, session_error};
use super::{DebugEngine, DebugStatus};
use crate::evaluation::parse_debug_expression;

impl DebugEngine {
    pub(super) fn evaluate(
        &mut self,
        request_id: u64,
        command: &str,
        source: String,
        frame_id: Option<u64>,
        async_eval: bool,
    ) -> Vec<DebugRecord> {
        if self.status != DebugStatus::Stopped {
            return vec![invalid_state(request_id, command, self.status)];
        }
        let limits = fpas_vm::DebugEvaluationLimits::default();
        let expression = match parse_debug_expression(&source, limits) {
            Ok(expression) => expression,
            Err(error) => return vec![parse_error(request_id, command, error)],
        };
        let Some(session) = self.actor.session_mut() else {
            return vec![invalid_state(request_id, command, self.status)];
        };
        if async_eval {
            let _ = session;
            self.pending_evaluation = Some((request_id, DebugCommand::from_name(command)));
            self.actor.evaluate(expression, frame_id, limits);
            return Vec::new();
        }
        match session.evaluate_with_limits(&expression, frame_id, limits) {
            Ok(result) => vec![ok(request_id, command, ResponseBody::Evaluate(result))],
            Err(error) => vec![session_error(request_id, command, error)],
        }
    }
}
