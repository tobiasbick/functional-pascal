//! Protocol-neutral debugger execution and asynchronous event delivery.

mod breakpoints;
mod command;
mod completed_result;
mod completion;
mod data_breakpoints;
mod dictionary;
mod dispatch;
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
mod runtime_failures;
mod sequence;
mod storage;
mod task_control;
mod tasks;
mod variant;

use std::collections::{HashMap, HashSet};

use serde_json::{Map, Value, json};

use crate::breakpoints::{BreakpointPolicy, RuntimeFailurePolicy};
use crate::jsonl::actor::{ResumeCommand, SessionActor};
use crate::jsonl::encode::*;
use crate::jsonl::protocol::{event, failure, session_error, success};
use crate::target::DebugReloadProvider;
use crate::{DebugSourceContent, PreparedDebugTarget};

pub(crate) use command::{DebugCommand, DebugRequest};
pub(crate) use record::DebugRecord;

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
    pending_evaluation: Option<(u64, String)>,
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
        if self.status == DebugStatus::Terminated {
            return vec![DebugRecord::from_jsonl(event(
                "protocol_error",
                error_body(
                    "invalid_state",
                    "The debugger session is terminated.",
                    "Start a new `fpas debug` process.",
                ),
            ))];
        }
        if !self.request_ids.insert(request.id) {
            return vec![DebugRecord::from_jsonl(failure(
                request.id,
                request.command.name(),
                "invalid_request",
                format!("Request ID {} was already used.", request.id),
                "Use a new positive request ID for every request.",
            ))];
        };
        let mut records =
            self.handle_request(request.id, request.command.name(), &request.arguments);
        records.extend(self.poll_values());
        records.into_iter().map(DebugRecord::from_jsonl).collect()
    }

    /// Poll for records caused by an asynchronous continue or step operation.
    #[must_use]
    pub(crate) fn poll(&mut self) -> Vec<DebugRecord> {
        self.poll_values()
            .into_iter()
            .map(DebugRecord::from_jsonl)
            .collect()
    }

    fn poll_values(&mut self) -> Vec<Value> {
        self.actor
            .poll()
            .map_or_else(Vec::new, |completion| self.complete_actor(completion))
    }

    /// Wait for the active resume operation and return its terminal or stopped events.
    #[must_use]
    pub(crate) fn wait(&mut self) -> Vec<DebugRecord> {
        self.wait_values()
            .into_iter()
            .map(DebugRecord::from_jsonl)
            .collect()
    }

    fn wait_values(&mut self) -> Vec<Value> {
        self.actor
            .wait()
            .map_or_else(Vec::new, |completion| self.complete_actor(completion))
    }

    /// Terminate after an adapter-level framing failure.
    pub(crate) fn fatal_protocol_error(&mut self, message: impl Into<String>) -> Vec<DebugRecord> {
        self.status = DebugStatus::Terminated;
        self.fatal_termination = true;
        vec![DebugRecord::from_jsonl(event(
            "protocol_error",
            error_body(
                "invalid_request",
                message,
                "Send one valid UTF-8 JSON request object per line.",
            ),
        ))]
    }

    fn initialize(
        &mut self,
        request_id: u64,
        command: &str,
        arguments: &Map<String, Value>,
    ) -> Vec<Value> {
        if self.status != DebugStatus::Created {
            return vec![invalid_state(request_id, command, self.status)];
        }
        let version = arguments
            .get("version")
            .and_then(Value::as_u64)
            .unwrap_or(2);
        if version != 2 {
            return vec![failure(
                request_id,
                command,
                "unsupported_protocol_version",
                format!("Protocol version {version} is unsupported."),
                "Request version 2.",
            )];
        }
        self.status = DebugStatus::Initialized;
        initialize_records(
            request_id,
            command,
            self.execution_limits,
            self.reloader.is_some(),
        )
    }

    fn launch(
        &mut self,
        request_id: u64,
        command: &str,
        arguments: &Map<String, Value>,
    ) -> Vec<Value> {
        if self.status != DebugStatus::Initialized {
            return vec![invalid_state(request_id, command, self.status)];
        }
        let stop_on_entry = arguments
            .get("stop_on_entry")
            .and_then(Value::as_bool)
            .unwrap_or(true);
        let mut records = vec![success(request_id, command, json!({"accepted": true}))];
        if stop_on_entry {
            self.status = DebugStatus::Stopped;
            if let Some(session) = self.actor.session() {
                records.push(stopped_event(session.last_stop()));
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

    fn resume(&mut self, request_id: u64, command: &str, resume: ResumeCommand) -> Vec<Value> {
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
                vec![success(request_id, command, json!({"accepted": true}))]
            }
            Err(error) => vec![session_error(request_id, command, error)],
        }
    }

    fn task_resume(
        &mut self,
        request_id: u64,
        command: &str,
        arguments: &Map<String, Value>,
        resume: fn(Option<u64>) -> ResumeCommand,
    ) -> Vec<Value> {
        match optional_u64_argument(request_id, command, arguments, "task_id") {
            Ok(task_id) => self.resume(request_id, command, resume(task_id)),
            Err(error) => vec![error],
        }
    }

    fn pause(&mut self, request_id: u64, command: &str) -> Vec<Value> {
        if self.status != DebugStatus::Running {
            return vec![invalid_state(request_id, command, self.status)];
        }
        self.actor.pause();
        vec![success(request_id, command, json!({"requested": true}))]
    }

    fn stack(
        &mut self,
        request_id: u64,
        command: &str,
        arguments: &Map<String, Value>,
    ) -> Vec<Value> {
        if self.status != DebugStatus::Stopped {
            return vec![invalid_state(request_id, command, self.status)];
        }
        let start = index_argument(arguments, "start", 0);
        let count = index_argument(arguments, "count", 64);
        let Some(session) = self.actor.session_mut() else {
            return vec![invalid_state(request_id, command, self.status)];
        };
        let task_id = match optional_u64_argument(request_id, command, arguments, "task_id") {
            Ok(task_id) => task_id.unwrap_or_else(|| session.last_stop().task_id),
            Err(error) => return vec![error],
        };
        match session.stack_for_task(task_id, start, count) {
            Ok(frames) => vec![success(
                request_id,
                command,
                json!({
                    "frames": frames.items.iter().map(frame_body).collect::<Vec<_>>(),
                    "total": frames.total,
                    "task_id": task_id
                }),
            )],
            Err(error) => vec![session_error(request_id, command, error)],
        }
    }

    fn scopes(
        &mut self,
        request_id: u64,
        command: &str,
        arguments: &Map<String, Value>,
    ) -> Vec<Value> {
        if self.status != DebugStatus::Stopped {
            return vec![invalid_state(request_id, command, self.status)];
        }
        let Some(frame_id) = arguments.get("frame_id").and_then(Value::as_u64) else {
            return vec![missing_argument(request_id, command, "frame_id")];
        };
        let Some(session) = self.actor.session() else {
            return vec![invalid_state(request_id, command, self.status)];
        };
        match session.scopes(frame_id) {
            Ok(scopes) => vec![success(
                request_id,
                command,
                json!({"scopes": scopes.iter().map(scope_body).collect::<Vec<_>>() }),
            )],
            Err(error) => vec![session_error(request_id, command, error)],
        }
    }

    fn variables(
        &mut self,
        request_id: u64,
        command: &str,
        arguments: &Map<String, Value>,
    ) -> Vec<Value> {
        if self.status != DebugStatus::Stopped {
            return vec![invalid_state(request_id, command, self.status)];
        }
        let Some(reference) = arguments.get("variables_reference").and_then(Value::as_u64) else {
            return vec![missing_argument(request_id, command, "variables_reference")];
        };
        let start = index_argument(arguments, "start", 0);
        let count = index_argument(arguments, "count", 100);
        let Some(session) = self.actor.session_mut() else {
            return vec![invalid_state(request_id, command, self.status)];
        };
        match session.variables(reference, start, count) {
            Ok(variables) => vec![success(
                request_id,
                command,
                json!({
                    "variables": variables.items.iter().map(variable_body).collect::<Vec<_>>(),
                    "total": variables.total
                }),
            )],
            Err(error) => vec![session_error(request_id, command, error)],
        }
    }

    fn disconnect(&mut self, request_id: u64, command: &str) -> Vec<Value> {
        if self.actor.is_evaluating() {
            self.actor.cancel_evaluation();
            let mut records = self.wait_values();
            records.extend(self.disconnect_session_events());
            records.push(success(request_id, command, json!({"terminated": true})));
            self.status = DebugStatus::Terminated;
            records.push(event(
                "terminated",
                json!({"reason": "disconnect", "exit_code": 0}),
            ));
            return records;
        }
        if self.status == DebugStatus::Running {
            self.actor.pause();
            let mut records = vec![success(request_id, command, json!({"terminated": true}))];
            records.extend(self.wait_values());
            records.extend(self.disconnect_session_events());
            self.status = DebugStatus::Terminated;
            records.push(event(
                "terminated",
                json!({"reason": "disconnect", "exit_code": 0}),
            ));
            return records;
        }
        let mut records = vec![success(request_id, command, json!({"terminated": true}))];
        records.extend(self.disconnect_session_events());
        self.status = DebugStatus::Terminated;
        records.push(event(
            "terminated",
            json!({"reason": "disconnect", "exit_code": 0}),
        ));
        records
    }

    fn disconnect_session_events(&mut self) -> Vec<Value> {
        let Some(session) = self.actor.session_mut() else {
            return Vec::new();
        };
        session.disconnect();
        session
            .take_task_events()
            .into_iter()
            .map(task_event)
            .collect()
    }

    fn cancel_evaluation(&mut self, request_id: u64, command: &str) -> Vec<Value> {
        let cancelled = self.actor.cancel_evaluation();
        vec![success(
            request_id,
            command,
            json!({"cancelled": cancelled}),
        )]
    }
}
