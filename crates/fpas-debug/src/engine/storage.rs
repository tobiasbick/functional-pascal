//! Seeded empty-storage descendant initialization.

use super::record::{DebugRecord, ResponseBody};
use super::reply::{invalid_state, ok, parse_error, session_error};
use super::{DebugEngine, DebugStatus};
use crate::evaluation::{parse_debug_assignment_target, parse_debug_expression};

impl DebugEngine {
    pub(super) fn initialize_storage(
        &mut self,
        request_id: u64,
        command: &str,
        target_source: String,
        initializer_source: String,
        expression_source: String,
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
        let initializer = match parse_debug_expression(&initializer_source, limits) {
            Ok(expression) => expression,
            Err(error) => return vec![parse_error(request_id, command, error)],
        };
        let expression = match parse_debug_expression(&expression_source, limits) {
            Ok(expression) => expression,
            Err(error) => return vec![parse_error(request_id, command, error)],
        };
        let Some(session) = self.actor.session_mut() else {
            return vec![invalid_state(request_id, command, self.status)];
        };
        match session.initialize_storage_with_limits(
            &target,
            &initializer,
            &expression,
            frame_id,
            limits,
        ) {
            Ok(result) => vec![ok(request_id, command, ResponseBody::Storage(result))],
            Err(error) => vec![session_error(request_id, command, error)],
        }
    }
}
