//! Completion handling between physical VM stops and logical debugger events.

use serde_json::{Value, json};

use super::{DebugEngine, DebugStatus};
use crate::breakpoints::BreakpointOutcome;
use crate::jsonl::actor::{ActorCompletion, EvaluationCompletion, ResumeCommand, ResumeCompletion};
use crate::jsonl::encode::{
    diagnostic_body, error_body, error_code, output_events, stopped_event, task_event,
};
use crate::jsonl::protocol::event;

impl DebugEngine {
    pub(super) fn complete_actor(&mut self, completion: ActorCompletion) -> Vec<Value> {
        match completion {
            ActorCompletion::Resume(completion) => self.complete_resume(completion),
            ActorCompletion::Evaluation(completion) => self.complete_evaluation(completion),
        }
    }

    fn complete_resume(&mut self, completion: ResumeCompletion) -> Vec<Value> {
        let ResumeCompletion {
            mut session,
            result,
        } = completion;
        let mut records = output_events(&session, &mut self.output_cursor);
        records.extend(session.take_task_events().into_iter().map(task_event));
        match result {
            Ok(fpas_vm::DebugRunResult::Stopped(stop)) => {
                self.status = DebugStatus::Stopped;
                if stop.reason == fpas_vm::DebugStopReason::RuntimeError
                    && let Some(diagnostic) = &stop.diagnostic
                {
                    records.push(event(
                        "runtime_error",
                        diagnostic_body(diagnostic, stop.task_id),
                    ));
                    if !self.runtime_failure_policy.should_stop(diagnostic.code) {
                        session.disconnect();
                        records.extend(session.take_task_events().into_iter().map(task_event));
                        self.status = DebugStatus::Terminated;
                        records.push(event(
                            "terminated",
                            json!({
                                "reason": "runtime_error",
                                "exit_code": 1,
                                "diagnostic_code": diagnostic.code.to_string()
                            }),
                        ));
                        return records;
                    }
                }
                if matches!(
                    stop.reason,
                    fpas_vm::DebugStopReason::Breakpoint | fpas_vm::DebugStopReason::DataBreakpoint
                ) && !stop.breakpoint_ids.is_empty()
                {
                    let frame_id = session
                        .stack(0, 1)
                        .ok()
                        .and_then(|frames| frames.items.first().map(|frame| frame.id));
                    if let Some(frame_id) = frame_id {
                        let mut should_stop = false;
                        for breakpoint_id in &stop.breakpoint_ids {
                            let Some(mut policy) = self.breakpoint_policies.remove(breakpoint_id)
                            else {
                                should_stop = true;
                                continue;
                            };
                            let remaining = crate::evaluation::LogMessageLimits::default()
                                .max_session_output_bytes
                                .saturating_sub(self.log_output_bytes);
                            let outcome = policy.apply(&mut session, frame_id, remaining);
                            self.breakpoint_policies.insert(*breakpoint_id, policy);
                            match outcome {
                                BreakpointOutcome::Stop => should_stop = true,
                                BreakpointOutcome::StopWithDiagnostic(diagnostic) => {
                                    should_stop = true;
                                    if let Some(diagnostic) = diagnostic {
                                        records.push(event(
                                            "protocol_error",
                                            error_body(
                                                error_code(diagnostic.kind),
                                                diagnostic.message,
                                                diagnostic.hint,
                                            ),
                                        ));
                                    }
                                }
                                BreakpointOutcome::Continue => {}
                                BreakpointOutcome::Log(output) => {
                                    self.log_output_bytes =
                                        self.log_output_bytes.saturating_add(output.len());
                                    records.push(event("output", json!({
                                        "category": "console",
                                        "text": output,
                                        "breakpoint_id": breakpoint_id,
                                        "location": stop.location.as_ref().map(|location| json!({
                                            "source": location.source,
                                            "line": location.line,
                                            "column": location.column
                                        }))
                                    })));
                                }
                                BreakpointOutcome::LogDiagnostic(diagnostic) => {
                                    if let Some(diagnostic) = diagnostic {
                                        records.push(event(
                                            "output",
                                            json!({
                                                "category": "stderr",
                                                "text": format!(
                                                    "Logpoint evaluation failed: {} Help: {}\n",
                                                    diagnostic.message, diagnostic.hint
                                                )
                                            }),
                                        ));
                                    }
                                }
                            }
                        }
                        if !should_stop {
                            self.actor.restore(session);
                            self.status = DebugStatus::Running;
                            if let Err(error) = self.actor.resume(ResumeCommand::Continue) {
                                self.status = DebugStatus::Stopped;
                                records.push(event(
                                    "protocol_error",
                                    error_body(error_code(error.kind), error.message, error.hint),
                                ));
                            }
                            return records;
                        }
                    }
                }
                records.push(stopped_event(&stop));
                self.actor.restore(session);
            }
            Ok(fpas_vm::DebugRunResult::Terminated(termination)) => {
                self.status = DebugStatus::Terminated;
                records.push(event(
                    "terminated",
                    json!({
                        "reason": "completed", "exit_code": 0,
                        "instruction_count": termination.instruction_count
                    }),
                ));
            }
            Err(error) => {
                self.status = DebugStatus::Stopped;
                records.push(event(
                    "protocol_error",
                    error_body(
                        error_code(error.kind),
                        error.message.clone(),
                        error.hint.clone(),
                    ),
                ));
                self.actor.restore(session);
            }
        }
        records
    }

    fn complete_evaluation(&mut self, completion: EvaluationCompletion) -> Vec<Value> {
        let EvaluationCompletion { session, result } = completion;
        self.actor.restore(session);
        let Some((request_id, command)) = self.pending_evaluation.take() else {
            return Vec::new();
        };
        match result {
            Ok(result) => vec![crate::jsonl::protocol::success(
                request_id,
                &command,
                json!({
                    "result": result.value,
                    "type_name": result.type_name,
                    "variables_reference": result.variables_reference,
                    "named_variables": result.named_variables,
                    "indexed_variables": result.indexed_variables
                }),
            )],
            Err(error) => vec![crate::jsonl::protocol::session_error(
                request_id, &command, error,
            )],
        }
    }
}
