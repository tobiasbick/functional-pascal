//! Protocol-neutral debugger execution and asynchronous event delivery.

pub(crate) mod actor;
mod breakpoints;
mod command;
mod completed_result;
mod completion;
mod data_breakpoints;
mod dictionary;
mod dispatch;
mod error;
mod evaluation;
mod forced_return;
mod frame_restart;
mod function_breakpoints;
mod instruction;
mod io;
mod lifecycle;
mod live_image;
mod location;
mod mutation;
mod record;
mod recording;
mod reply;
mod request;
mod runtime_failures;
mod sequence;
mod storage;
mod task_control;
mod tasks;
mod variant;

use std::collections::{HashMap, HashSet};

use crate::breakpoints::{BreakpointPolicy, RuntimeFailurePolicy};
use crate::target::DebugReloadProvider;
use crate::{DebugSourceContent, PreparedDebugTarget};

use self::actor::{ResumeCommand, SessionActor};
use self::reply::{event, fail, invalid_state, ok, session_error};

pub(crate) use command::DebugCommand;
pub(crate) use error::EngineFailure;
pub(crate) use record::{DebugEvent, DebugRecord, ResponseBody};
pub(crate) use request::{AssignOp, DataBreakpointOp, DebugOp, DebugRequest, FunctionBreakpointOp};

#[cfg(test)]
mod tests;

/// Coarse debugger lifecycle visible to protocol adapters.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DebugStatus {
    /// Waiting for `initialize`.
    Created,
    /// Initialized but not launched.
    Initialized,
    /// Program execution is active.
    Running,
    /// Program execution is stopped and inspectable.
    Stopped,
    /// Session ended or a fatal protocol error occurred.
    Terminated,
}

/// Protocol-neutral state and execution engine for one prepared target.
pub(crate) struct DebugEngine {
    status: DebugStatus,
    fatal_termination: bool,
    actor: SessionActor,
    execution_limits: fpas_vm::DebugExecutionLimits,
    request_ids: HashSet<u64>,
    output_cursor: usize,
    breakpoint_policies: HashMap<u64, BreakpointPolicy>,
    function_breakpoint_ids: Vec<u64>,
    data_breakpoint_ids: Vec<u64>,
    runtime_failure_policy: RuntimeFailurePolicy,
    log_output_bytes: usize,
    pending_evaluation: Option<(u64, DebugCommand)>,
    reloader: Option<DebugReloadProvider>,
    sources: Vec<DebugSourceContent>,
    previous_sources: Option<Vec<DebugSourceContent>>,
    source_revision: u64,
}

impl DebugEngine {
    /// Construct a server around one prepared target.
    ///
    /// # Errors
    ///
    /// Returns a debugger initialization error for invalid runtime state.
    pub(crate) fn new(mut target: PreparedDebugTarget) -> Result<Self, fpas_vm::DebugSessionError> {
        let execution_limits = target.execution_limits();
        let reloader = target.take_reloader();
        let sources = target.sources().to_vec();
        Ok(Self {
            status: DebugStatus::Created,
            fatal_termination: false,
            actor: SessionActor::new(target.into_session()?),
            execution_limits,
            request_ids: HashSet::new(),
            output_cursor: 0,
            breakpoint_policies: HashMap::new(),
            function_breakpoint_ids: Vec::new(),
            data_breakpoint_ids: Vec::new(),
            runtime_failure_policy: RuntimeFailurePolicy::default(),
            log_output_bytes: 0,
            pending_evaluation: None,
            reloader,
            sources,
            previous_sources: None,
            source_revision: 1,
        })
    }

    /// Return the current debugger lifecycle state.
    #[must_use]
    pub(crate) const fn status(&self) -> DebugStatus {
        self.status
    }

    pub(crate) const fn terminated_fatally(&self) -> bool {
        self.fatal_termination
    }

    /// Whether a detached call evaluation currently owns the session actor.
    #[must_use]
    pub(crate) fn is_evaluating(&self) -> bool {
        self.actor.is_evaluating()
    }

    pub(crate) const fn supports_hot_reload(&self) -> bool {
        self.reloader.is_some()
    }

    pub(crate) const fn source_revision(&self) -> u64 {
        self.source_revision
    }

    pub(crate) fn sources(&self) -> &[DebugSourceContent] {
        &self.sources
    }

    /// Execute one validated typed request.
    #[must_use]
    pub(crate) fn execute(&mut self, request: DebugRequest) -> Vec<DebugRecord> {
        let command = request.command();
        if self.status == DebugStatus::Terminated {
            return vec![event(DebugEvent::ProtocolError(
                EngineFailure::terminated_session(),
            ))];
        }
        if !self.request_ids.insert(request.id) {
            return vec![fail(
                request.id,
                command.name(),
                EngineFailure::duplicate_request(request.id),
            )];
        }
        let mut records = self.handle_request(request.id, request.op);
        records.extend(self.poll_values());
        records
    }

    /// Poll for records caused by an asynchronous continue or step operation.
    #[must_use]
    pub(crate) fn poll(&mut self) -> Vec<DebugRecord> {
        self.poll_values()
    }

    fn poll_values(&mut self) -> Vec<DebugRecord> {
        self.actor
            .poll()
            .map_or_else(Vec::new, |completion| self.complete_actor(completion))
    }

    /// Wait for the active resume operation and return its terminal or stopped events.
    #[must_use]
    pub(crate) fn wait(&mut self) -> Vec<DebugRecord> {
        self.wait_values()
    }

    fn wait_values(&mut self) -> Vec<DebugRecord> {
        self.actor
            .wait()
            .map_or_else(Vec::new, |completion| self.complete_actor(completion))
    }

    /// Terminate after an adapter-level framing failure.
    pub(crate) fn fatal_protocol_error(&mut self, message: impl Into<String>) -> Vec<DebugRecord> {
        self.status = DebugStatus::Terminated;
        self.fatal_termination = true;
        vec![event(DebugEvent::ProtocolError(EngineFailure::new(
            "invalid_request",
            message,
            "Send one valid UTF-8 JSON request object per line.",
        )))]
    }

    fn initialize(&mut self, request_id: u64, command: &str) -> Vec<DebugRecord> {
        if self.status != DebugStatus::Created {
            return vec![invalid_state(request_id, command, self.status)];
        }
        self.status = DebugStatus::Initialized;
        vec![
            ok(
                request_id,
                command,
                ResponseBody::Initialize {
                    execution: self.execution_limits,
                    hot_reload: self.reloader.is_some(),
                },
            ),
            event(DebugEvent::Initialized),
        ]
    }

    fn launch(&mut self, request_id: u64, command: &str, stop_on_entry: bool) -> Vec<DebugRecord> {
        if self.status != DebugStatus::Initialized {
            return vec![invalid_state(request_id, command, self.status)];
        }
        let mut records = vec![ok(request_id, command, ResponseBody::Accepted)];
        if stop_on_entry {
            self.status = DebugStatus::Stopped;
            if let Some(session) = self.actor.session() {
                records.push(event(DebugEvent::Stopped(session.last_stop().clone())));
            }
        } else {
            self.status = DebugStatus::Running;
            if let Err(error) = self.actor.resume(ResumeCommand::Continue) {
                self.status = DebugStatus::Stopped;
                records.push(session_error(request_id, command, error));
            }
        }
        records
    }

    fn resume(
        &mut self,
        request_id: u64,
        command: &str,
        resume: ResumeCommand,
    ) -> Vec<DebugRecord> {
        if self.status != DebugStatus::Stopped {
            return vec![invalid_state(request_id, command, self.status)];
        }
        if self.actor.session().is_none() {
            return vec![invalid_state(request_id, command, self.status)];
        }
        if let Some(task_id) = resume.task_id()
            && let Some(session) = self.actor.session_mut()
            && let Err(error) = session.select_task(task_id)
        {
            return vec![session_error(request_id, command, error)];
        }
        match self.actor.resume(resume) {
            Ok(()) => {
                self.status = DebugStatus::Running;
                vec![ok(request_id, command, ResponseBody::Accepted)]
            }
            Err(error) => vec![session_error(request_id, command, error)],
        }
    }

    fn pause(&mut self, request_id: u64, command: &str) -> Vec<DebugRecord> {
        if self.status != DebugStatus::Running {
            return vec![invalid_state(request_id, command, self.status)];
        }
        self.actor.pause();
        vec![ok(request_id, command, ResponseBody::Requested)]
    }

    fn stack(
        &mut self,
        request_id: u64,
        command: &str,
        start: usize,
        count: usize,
        task_id: Option<u64>,
    ) -> Vec<DebugRecord> {
        if self.status != DebugStatus::Stopped {
            return vec![invalid_state(request_id, command, self.status)];
        }
        let Some(session) = self.actor.session_mut() else {
            return vec![invalid_state(request_id, command, self.status)];
        };
        let task_id = task_id.unwrap_or_else(|| session.last_stop().task_id);
        match session.stack_for_task(task_id, start, count) {
            Ok(frames) => vec![ok(
                request_id,
                command,
                ResponseBody::Stack {
                    frames: frames.items,
                    total: frames.total,
                    task_id,
                },
            )],
            Err(error) => vec![session_error(request_id, command, error)],
        }
    }

    fn scopes(&mut self, request_id: u64, command: &str, frame_id: u64) -> Vec<DebugRecord> {
        if self.status != DebugStatus::Stopped {
            return vec![invalid_state(request_id, command, self.status)];
        }
        let Some(session) = self.actor.session() else {
            return vec![invalid_state(request_id, command, self.status)];
        };
        match session.scopes(frame_id) {
            Ok(scopes) => vec![ok(request_id, command, ResponseBody::Scopes { scopes })],
            Err(error) => vec![session_error(request_id, command, error)],
        }
    }

    fn variables(
        &mut self,
        request_id: u64,
        command: &str,
        reference: u64,
        start: usize,
        count: usize,
    ) -> Vec<DebugRecord> {
        if self.status != DebugStatus::Stopped {
            return vec![invalid_state(request_id, command, self.status)];
        }
        let Some(session) = self.actor.session_mut() else {
            return vec![invalid_state(request_id, command, self.status)];
        };
        match session.variables(reference, start, count) {
            Ok(variables) => vec![ok(
                request_id,
                command,
                ResponseBody::Variables {
                    variables: variables.items,
                    total: variables.total,
                },
            )],
            Err(error) => vec![session_error(request_id, command, error)],
        }
    }

    fn disconnect(&mut self, request_id: u64, command: &str) -> Vec<DebugRecord> {
        let terminated = event(DebugEvent::Terminated {
            reason: "disconnect",
            exit_code: 0,
            diagnostic_code: None,
            instruction_count: None,
        });
        if self.actor.is_evaluating() {
            self.actor.cancel_evaluation();
            let mut records = self.wait_values();
            records.extend(self.disconnect_session_events());
            records.push(ok(request_id, command, ResponseBody::TerminatedAck));
            self.status = DebugStatus::Terminated;
            records.push(terminated);
            return records;
        }
        if self.status == DebugStatus::Running {
            self.actor.pause();
            let mut records = vec![ok(request_id, command, ResponseBody::TerminatedAck)];
            records.extend(self.wait_values());
            records.extend(self.disconnect_session_events());
            self.status = DebugStatus::Terminated;
            records.push(terminated);
            return records;
        }
        let mut records = vec![ok(request_id, command, ResponseBody::TerminatedAck)];
        records.extend(self.disconnect_session_events());
        self.status = DebugStatus::Terminated;
        records.push(terminated);
        records
    }

    fn disconnect_session_events(&mut self) -> Vec<DebugRecord> {
        let Some(session) = self.actor.session_mut() else {
            return Vec::new();
        };
        session.disconnect();
        session
            .take_task_events()
            .into_iter()
            .map(crate::engine::reply::task_event)
            .collect()
    }

    fn cancel_evaluation(&mut self, request_id: u64, command: &str) -> Vec<DebugRecord> {
        let cancelled = self.actor.cancel_evaluation();
        vec![ok(
            request_id,
            command,
            ResponseBody::Cancelled { cancelled },
        )]
    }
}
