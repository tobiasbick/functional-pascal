//! Variant discovery and complete construction.

use super::record::{DebugRecord, ResponseBody};
use super::reply::{invalid_state, ok, parse_error, session_error};
use super::{DebugEngine, DebugStatus};
use crate::evaluation::{parse_debug_assignment_target, parse_debug_expression};

impl DebugEngine {
    pub(super) fn describe_variant(
        &mut self,
        request_id: u64,
        command: &str,
        target_source: String,
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
        let Some(session) = self.actor.session_mut() else {
            return vec![invalid_state(request_id, command, self.status)];
        };
        match session.describe_variant_with_limits(&target, frame_id, limits) {
            Ok(description) => vec![ok(
                request_id,
                command,
                ResponseBody::VariantDescription {
                    target: target_source,
                    description,
                },
            )],
            Err(error) => vec![session_error(request_id, command, error)],
        }
    }

    pub(super) fn construct_variant(
        &mut self,
        request_id: u64,
        command: &str,
        target_source: String,
        variant: String,
        fields: Vec<(String, String)>,
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
        let mut parsed_fields = Vec::with_capacity(fields.len());
        for (name, source) in fields {
            match parse_debug_expression(&source, limits) {
                Ok(expression) => parsed_fields.push((name, expression)),
                Err(error) => return vec![parse_error(request_id, command, error)],
            }
        }
        let Some(session) = self.actor.session_mut() else {
            return vec![invalid_state(request_id, command, self.status)];
        };
        match session.construct_variant_with_limits(
            &target,
            &variant,
            &parsed_fields,
            frame_id,
            limits,
        ) {
            Ok(result) => vec![ok(
                request_id,
                command,
                ResponseBody::VariantConstruct(result),
            )],
            Err(error) => vec![session_error(request_id, command, error)],
        }
    }
}
