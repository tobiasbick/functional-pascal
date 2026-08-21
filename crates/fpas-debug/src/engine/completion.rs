//! Completion handling between physical VM stops and logical debugger events.

use super::actor::{ActorCompletion, EvaluationCompletion, ResumeCommand, ResumeCompletion};
use super::error::EngineFailure;
use super::record::{DebugEvent, DebugRecord, ResponseBody};
use super::reply::{event, ok, output_events, session_error, task_event};
use super::{DebugEngine, DebugStatus};
use crate::breakpoints::BreakpointOutcome;

impl DebugEngine {
    pub(super) fn complete_actor(&mut self, completion: ActorCompletion) -> Vec<DebugRecord> {
        match completion {
            ActorCompletion::Resume(completion) => self.complete_resume(completion),
            ActorCompletion::Evaluation(completion) => self.complete_evaluation(completion),
        }
    }

    fn complete_resume(&mut self, completion: ResumeCompletion) -> Vec<DebugRecord> {
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
                    records.push(event(DebugEvent::RuntimeError {
                        diagnostic: diagnostic.clone(),
                        task_id: stop.task_id,
                    }));
                    if !self.runtime_failure_policy.should_stop(diagnostic.code) {
                        session.disconnect();
                        records.extend(session.take_task_events().into_iter().map(task_event));
                        self.status = DebugStatus::Terminated;
                        records.push(event(DebugEvent::Terminated {
                            reason: "runtime_error",
                            exit_code: 1,
                            diagnostic_code: Some(diagnostic.code.to_string()),
                            instruction_count: None,
                        }));
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
                                        records.push(event(DebugEvent::ProtocolError(
                                            EngineFailure::from_session(diagnostic),
                                        )));
                                    }
                                }
                                BreakpointOutcome::Continue => {}
                                BreakpointOutcome::Log(output) => {
                                    self.log_output_bytes =
                                        self.log_output_bytes.saturating_add(output.len());
                                    records.push(event(DebugEvent::Output {
                                        category: "console",
                                        text: output,
                                        sequence: None,
                                        breakpoint_id: Some(*breakpoint_id),
                                        location: stop.location.clone(),
                                    }));
                                }
                                BreakpointOutcome::LogDiagnostic(diagnostic) => {
                                    if let Some(diagnostic) = diagnostic {
                                        records.push(event(DebugEvent::Output {
                                            category: "stderr",
                                            text: format!(
                                                "Logpoint evaluation failed: {} Help: {}\n",
                                                diagnostic.message, diagnostic.hint
                                            ),
                                            sequence: None,
                                            breakpoint_id: None,
                                            location: None,
                                        }));
                                    }
                                }
                            }
                        }
                        if !should_stop {
                            self.actor.restore(session);
                            self.status = DebugStatus::Running;
                            if let Err(error) = self.actor.resume(ResumeCommand::Continue) {
                                self.status = DebugStatus::Stopped;
                                records.push(event(DebugEvent::ProtocolError(
                                    EngineFailure::from_session(error),
                                )));
                            }
                            return records;
                        }
                    }
                }
                records.push(event(DebugEvent::Stopped(stop)));
                self.actor.restore(session);
            }
            Ok(fpas_vm::DebugRunResult::Terminated(termination)) => {
                self.status = DebugStatus::Terminated;
                records.push(event(DebugEvent::Terminated {
                    reason: "completed",
                    exit_code: 0,
                    diagnostic_code: None,
                    instruction_count: Some(termination.instruction_count),
                }));
            }
            Err(error) => {
                self.status = DebugStatus::Stopped;
                records.push(event(DebugEvent::ProtocolError(
                    EngineFailure::from_session(error.clone()),
                )));
                self.actor.restore(session);
            }
        }
        records
    }

    fn complete_evaluation(&mut self, completion: EvaluationCompletion) -> Vec<DebugRecord> {
        let EvaluationCompletion { session, result } = completion;
        self.actor.restore(session);
        let Some((request_id, command)) = self.pending_evaluation.take() else {
            return Vec::new();
        };
        match result {
            Ok(result) => vec![ok(
                request_id,
                command.name(),
                ResponseBody::Evaluate(result),
            )],
            Err(error) => vec![session_error(request_id, command.name(), error)],
        }
    }
}
