//! Replacement of one retained completed task result.

use super::record::{DebugRecord, ResponseBody};
use super::reply::{invalid_state, ok, parse_error, session_error};
use super::{DebugEngine, DebugStatus};
use crate::evaluation::parse_debug_expression;

impl DebugEngine {
    pub(super) fn replace_completed_task_result(
        &mut self,
        request_id: u64,
        command: &str,
        task_id: u64,
        expression_source: Option<String>,
        frame_id: Option<u64>,
    ) -> Vec<DebugRecord> {
        if self.status != DebugStatus::Stopped {
            return vec![invalid_state(request_id, command, self.status)];
        }
        let limits = fpas_vm::DebugEvaluationLimits::default();
        let expression = match expression_source {
            Some(source) => match parse_debug_expression(&source, limits) {
                Ok(expression) => Some(expression),
                Err(error) => return vec![parse_error(request_id, command, error)],
            },
            None => None,
        };
        let Some(session) = self.actor.session_mut() else {
            return vec![invalid_state(request_id, command, self.status)];
        };
        match session.replace_completed_task_result_with_limits(
            task_id,
            frame_id,
            expression.as_ref(),
            limits,
        ) {
            Ok(result) => vec![ok(request_id, command, ResponseBody::TaskResult(result))],
            Err(error) => vec![session_error(request_id, command, error)],
        }
    }
}
