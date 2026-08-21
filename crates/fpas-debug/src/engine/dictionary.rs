//! Atomic dictionary structure mutation.

use super::record::{DebugRecord, ResponseBody};
use super::reply::{invalid_state, ok, parse_error, session_error};
use super::{DebugEngine, DebugStatus};
use crate::evaluation::{parse_debug_assignment_target, parse_debug_expression};

impl DebugEngine {
    pub(super) fn insert_dictionary(
        &mut self,
        request_id: u64,
        command: &str,
        target: String,
        key: String,
        expression: String,
        frame_id: Option<u64>,
    ) -> Vec<DebugRecord> {
        self.mutate_dictionary(
            request_id,
            command,
            target,
            vec![key, expression],
            frame_id,
            |session, target, expressions, frame_id, limits| {
                session.insert_dictionary_entry_with_limits(
                    target,
                    &expressions[0],
                    &expressions[1],
                    frame_id,
                    limits,
                )
            },
        )
    }

    pub(super) fn remove_dictionary(
        &mut self,
        request_id: u64,
        command: &str,
        target: String,
        key: String,
        frame_id: Option<u64>,
    ) -> Vec<DebugRecord> {
        self.mutate_dictionary(
            request_id,
            command,
            target,
            vec![key],
            frame_id,
            |session, target, expressions, frame_id, limits| {
                session.remove_dictionary_entry_with_limits(
                    target,
                    &expressions[0],
                    frame_id,
                    limits,
                )
            },
        )
    }

    pub(super) fn replace_dictionary_key(
        &mut self,
        request_id: u64,
        command: &str,
        target: String,
        key: String,
        new_key: String,
        frame_id: Option<u64>,
    ) -> Vec<DebugRecord> {
        self.mutate_dictionary(
            request_id,
            command,
            target,
            vec![key, new_key],
            frame_id,
            |session, target, expressions, frame_id, limits| {
                session.replace_dictionary_key_with_limits(
                    target,
                    &expressions[0],
                    &expressions[1],
                    frame_id,
                    limits,
                )
            },
        )
    }

    fn mutate_dictionary(
        &mut self,
        request_id: u64,
        command: &str,
        target_source: String,
        expression_sources: Vec<String>,
        frame_id: Option<u64>,
        operate: impl FnOnce(
            &mut fpas_vm::DebugSession,
            &fpas_vm::DebugAssignmentTarget,
            &[fpas_vm::DebugExpression],
            Option<u64>,
            fpas_vm::DebugEvaluationLimits,
        ) -> Result<
            fpas_vm::DebugDictionaryMutationResult,
            fpas_vm::DebugSessionError,
        >,
    ) -> Vec<DebugRecord> {
        if self.status != DebugStatus::Stopped {
            return vec![invalid_state(request_id, command, self.status)];
        }
        let limits = fpas_vm::DebugEvaluationLimits::default();
        let target = match parse_debug_assignment_target(&target_source, limits) {
            Ok(target) => target,
            Err(error) => return vec![parse_error(request_id, command, error)],
        };
        let mut expressions = Vec::with_capacity(expression_sources.len());
        for source in &expression_sources {
            match parse_debug_expression(source, limits) {
                Ok(expression) => expressions.push(expression),
                Err(error) => return vec![parse_error(request_id, command, error)],
            }
        }
        let Some(session) = self.actor.session_mut() else {
            return vec![invalid_state(request_id, command, self.status)];
        };
        match operate(session, &target, &expressions, frame_id, limits) {
            Ok(result) => vec![ok(request_id, command, ResponseBody::Dictionary(result))],
            Err(error) => vec![session_error(request_id, command, error)],
        }
    }
}
