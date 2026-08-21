//! Atomic stopped-state variable mutation.

use super::record::{DebugRecord, ResponseBody};
use super::reply::{invalid_state, ok, parse_error, session_error};
use super::{DebugEngine, DebugStatus};
use crate::evaluation::{parse_debug_assignment_target, parse_debug_expression};

impl DebugEngine {
    pub(super) fn set_variable(
        &mut self,
        request_id: u64,
        command: &str,
        variables_reference: u64,
        name: String,
        source: String,
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
        match session.set_variable_with_limits(variables_reference, &name, &expression, limits) {
            Ok(result) => vec![ok(request_id, command, ResponseBody::Evaluate(result))],
            Err(error) => vec![session_error(request_id, command, error)],
        }
    }

    pub(super) fn set_expression(
        &mut self,
        request_id: u64,
        command: &str,
        target_source: String,
        replacement_source: String,
        frame_id: Option<u64>,
    ) -> Vec<DebugRecord> {
        if self.status != DebugStatus::Stopped {
            return vec![invalid_state(request_id, command, self.status)];
        }
        let limits = fpas_vm::DebugEvaluationLimits::default();
        let target = match parse_debug_assignment_target(&target_source, limits) {
            Ok(target) => target,
            Err(error) => return vec![parse_error(request_id, command, error)],
        };
        let replacement = match parse_debug_expression(&replacement_source, limits) {
            Ok(expression) => expression,
            Err(error) => return vec![parse_error(request_id, command, error)],
        };
        let Some(session) = self.actor.session_mut() else {
            return vec![invalid_state(request_id, command, self.status)];
        };
        match session.set_expression_with_limits(&target, &replacement, frame_id, limits) {
            Ok(result) => vec![ok(request_id, command, ResponseBody::Evaluate(result))],
            Err(error) => vec![session_error(request_id, command, error)],
        }
    }
}
