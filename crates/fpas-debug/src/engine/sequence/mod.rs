//! Shared parsing for sequence structure mutation.

mod array;
mod string;

use super::DebugStatus;
use super::reply::parse_error;
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
) -> Result<SequenceRequest, Box<super::record::DebugRecord>> {
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
