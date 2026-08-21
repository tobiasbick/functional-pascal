//! Forced return from a selected live frame.

use super::record::{DebugEvent, DebugRecord, ResponseBody};
use super::reply::{event, invalid_state, ok, parse_error, session_error};
use super::{DebugEngine, DebugStatus};
use crate::evaluation::parse_debug_expression;

impl DebugEngine {
    pub(super) fn force_return(
        &mut self,
        request_id: u64,
        command: &str,
        frame_id: u64,
        expression_source: Option<String>,
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
        match session.force_return_with_limits(frame_id, expression.as_ref(), limits) {
            Ok(result) => {
                let terminated = result.terminated;
                let mut records = vec![ok(request_id, command, ResponseBody::ForcedReturn(result))];
                if terminated {
                    self.status = DebugStatus::Terminated;
                    records.push(event(DebugEvent::Terminated {
                        reason: "completed",
                        exit_code: 0,
                        diagnostic_code: None,
                        instruction_count: None,
                    }));
                }
                records
            }
            Err(error) => vec![session_error(request_id, command, error)],
        }
    }
}
