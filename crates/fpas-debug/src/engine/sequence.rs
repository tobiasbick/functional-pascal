//! Array insertion/removal and string character replacement requests.

use fpas_vm::{DebugSession, DebugSessionError};

use super::record::{DebugRecord, ResponseBody};
use super::reply::{invalid_state, ok, parse_error, session_error};
use super::{DebugEngine, DebugStatus};
use crate::evaluation::{parse_debug_assignment_target, parse_debug_expression};

struct SequenceRequest {
    target: fpas_vm::DebugAssignmentTarget,
    expressions: Vec<fpas_vm::DebugExpression>,
    frame_id: Option<u64>,
    limits: fpas_vm::DebugEvaluationLimits,
}

fn parse_request(
    request_id: u64,
    command: &str,
    status: DebugStatus,
    target_source: &str,
    expression_sources: &[String],
    frame_id: Option<u64>,
) -> Result<SequenceRequest, Box<DebugRecord>> {
    if status != DebugStatus::Stopped {
        return Err(Box::new(super::reply::invalid_state(
            request_id, command, status,
        )));
    }
    let limits = fpas_vm::DebugEvaluationLimits::default();
    let target = parse_debug_assignment_target(target_source, limits)
        .map_err(|error| Box::new(parse_error(request_id, command, error)))?;
    let mut expressions = Vec::with_capacity(expression_sources.len());
    for source in expression_sources {
        expressions.push(
            parse_debug_expression(source, limits)
                .map_err(|error| Box::new(parse_error(request_id, command, error)))?,
        );
    }
    Ok(SequenceRequest {
        target,
        expressions,
        frame_id,
        limits,
    })
}

impl DebugEngine {
    pub(in crate::engine) fn insert_array(
        &mut self,
        request_id: u64,
        command: &str,
        target: String,
        index: String,
        expression: String,
        frame_id: Option<u64>,
    ) -> Vec<DebugRecord> {
        self.run_sequence_mutation(
            request_id,
            command,
            &target,
            &[index, expression],
            frame_id,
            |session, request| {
                session.insert_array_element_with_limits(
                    &request.target,
                    &request.expressions[0],
                    &request.expressions[1],
                    request.frame_id,
                    request.limits,
                )
            },
            ResponseBody::Array,
        )
    }

    pub(in crate::engine) fn remove_array(
        &mut self,
        request_id: u64,
        command: &str,
        target: String,
        index: String,
        frame_id: Option<u64>,
    ) -> Vec<DebugRecord> {
        self.run_sequence_mutation(
            request_id,
            command,
            &target,
            &[index],
            frame_id,
            |session, request| {
                session.remove_array_element_with_limits(
                    &request.target,
                    &request.expressions[0],
                    request.frame_id,
                    request.limits,
                )
            },
            ResponseBody::Array,
        )
    }

    pub(in crate::engine) fn replace_string_character(
        &mut self,
        request_id: u64,
        command: &str,
        target: String,
        index: String,
        expression: String,
        frame_id: Option<u64>,
    ) -> Vec<DebugRecord> {
        self.run_sequence_mutation(
            request_id,
            command,
            &target,
            &[index, expression],
            frame_id,
            |session, request| {
                session.replace_string_character_with_limits(
                    &request.target,
                    &request.expressions[0],
                    &request.expressions[1],
                    request.frame_id,
                    request.limits,
                )
            },
            ResponseBody::StringCharacter,
        )
    }

    /// Parse a sequence request, apply `mutate` to the stopped session, and wrap the reply.
    #[expect(
        clippy::too_many_arguments,
        reason = "request identity, sources, and the mutation callbacks are all required"
    )]
    fn run_sequence_mutation<T>(
        &mut self,
        request_id: u64,
        command: &str,
        target: &str,
        expression_sources: &[String],
        frame_id: Option<u64>,
        mutate: impl FnOnce(&mut DebugSession, &SequenceRequest) -> Result<T, DebugSessionError>,
        body: fn(T) -> ResponseBody,
    ) -> Vec<DebugRecord> {
        let request = match parse_request(
            request_id,
            command,
            self.status,
            target,
            expression_sources,
            frame_id,
        ) {
            Ok(request) => request,
            Err(response) => return vec![*response],
        };
        let Some(session) = self.actor.session_mut() else {
            return vec![invalid_state(request_id, command, self.status)];
        };
        match mutate(session, &request) {
            Ok(result) => vec![ok(request_id, command, body(result))],
            Err(error) => vec![session_error(request_id, command, error)],
        }
    }
}
