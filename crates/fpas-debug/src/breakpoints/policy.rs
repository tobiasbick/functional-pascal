//! Deterministic condition, physical-hit, assign, and log-or-stop policy.

use std::collections::HashSet;

use fpas_vm::{DebugEvaluationLimits, DebugExpression, DebugSession, DebugSessionError};

use super::BreakpointAssign;
use super::hit_condition;
use crate::evaluation::{
    EvaluationParseError, LogMessage, LogMessageLimits, LogSegment, parse_debug_expression,
};

#[derive(Debug, Clone)]
pub(crate) struct BreakpointPolicy {
    condition: Option<DebugExpression>,
    hit_condition: Option<u64>,
    log_message: Option<LogMessage>,
    assign: Option<BreakpointAssign>,
    physical_hits: u64,
    reported_errors: HashSet<String>,
}

impl BreakpointPolicy {
    pub(crate) fn parse(
        condition: Option<&str>,
        hit_condition_source: Option<&str>,
        log_message_source: Option<&str>,
        assign: Option<BreakpointAssign>,
    ) -> Result<Self, EvaluationParseError> {
        let evaluation_limits = DebugEvaluationLimits::default();
        let condition = condition
            .filter(|source| !source.is_empty())
            .map(|source| parse_debug_expression(source, evaluation_limits))
            .transpose()?;
        let hit_condition = hit_condition_source.map(hit_condition::parse).transpose()?;
        let log_message = log_message_source
            .filter(|source| !source.is_empty())
            .map(|source| LogMessage::parse(source, LogMessageLimits::default(), evaluation_limits))
            .transpose()?;
        Ok(Self {
            condition,
            hit_condition,
            log_message,
            assign,
            physical_hits: 0,
            reported_errors: HashSet::new(),
        })
    }

    /// Policy that only assigns one global, then stops.
    pub(crate) fn with_assign(assign: BreakpointAssign) -> Self {
        Self {
            condition: None,
            hit_condition: None,
            log_message: None,
            assign: Some(assign),
            physical_hits: 0,
            reported_errors: HashSet::new(),
        }
    }

    pub(crate) fn apply(
        &mut self,
        session: &mut DebugSession,
        frame_id: u64,
        remaining_log_bytes: usize,
    ) -> BreakpointOutcome {
        self.physical_hits = self.physical_hits.saturating_add(1);
        if let Some(condition) = &self.condition {
            match session.evaluate_boolean(condition, Some(frame_id)) {
                Ok(false) => return BreakpointOutcome::Continue,
                Ok(true) => {}
                Err(error) => {
                    return BreakpointOutcome::StopWithDiagnostic(self.once_diagnostic(error));
                }
            }
        }
        if self
            .hit_condition
            .is_some_and(|expected| expected != self.physical_hits)
        {
            return BreakpointOutcome::Continue;
        }
        let mut frame_id = frame_id;
        if let Some(assign) = &self.assign {
            match session.assign_data_location_in_frame(
                assign.identity,
                &assign.expression,
                Some(frame_id),
            ) {
                Ok(_) => {
                    if let Some(id) = session
                        .stack(0, 1)
                        .ok()
                        .and_then(|page| page.items.first().map(|frame| frame.id))
                    {
                        frame_id = id;
                    }
                }
                Err(error) => {
                    return BreakpointOutcome::StopWithDiagnostic(self.once_diagnostic(error));
                }
            }
        }
        let Some(log_message) = &self.log_message else {
            return BreakpointOutcome::Stop;
        };
        match render_log_message(log_message, session, frame_id, remaining_log_bytes) {
            Ok(output) => BreakpointOutcome::Log(output),
            Err(error) => BreakpointOutcome::LogDiagnostic(self.once_diagnostic(error)),
        }
    }

    fn once_diagnostic(&mut self, error: DebugSessionError) -> Option<DebugSessionError> {
        let identity = format!("{:?}:{}", error.kind, error.message);
        self.reported_errors.insert(identity).then_some(error)
    }
}

pub(crate) enum BreakpointOutcome {
    Stop,
    StopWithDiagnostic(Option<DebugSessionError>),
    Continue,
    Log(String),
    LogDiagnostic(Option<DebugSessionError>),
}

fn render_log_message(
    message: &LogMessage,
    session: &mut DebugSession,
    frame_id: u64,
    remaining_log_bytes: usize,
) -> Result<String, DebugSessionError> {
    let mut output = String::new();
    for segment in message.segments() {
        match segment {
            LogSegment::Text(text) => output.push_str(text),
            LogSegment::Expression(expression) => {
                output.push_str(&session.evaluate(expression, Some(frame_id))?.value)
            }
        }
        if output.len() > remaining_log_bytes {
            return Err(DebugSessionError {
                kind: fpas_vm::DebugErrorKind::EvaluationLimit,
                message: "logpoint output exceeds the remaining session output limit".to_string(),
                hint: "Use a shorter log message or fewer loop hits.".to_string(),
            });
        }
    }
    output.push('\n');
    if output.len() > remaining_log_bytes {
        return Err(DebugSessionError {
            kind: fpas_vm::DebugErrorKind::EvaluationLimit,
            message: "logpoint output exceeds the remaining session output limit".to_string(),
            hint: "Use a shorter log message or fewer loop hits.".to_string(),
        });
    }
    Ok(output)
}
