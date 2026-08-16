//! Stateful JSONL request handling and asynchronous execution event delivery.

mod breakpoints;
mod completed_result;
mod completion;
mod dictionary;
mod dispatch;
mod evaluation;
mod forced_return;
mod frame_restart;
mod function_breakpoints;
mod mutation;
mod runtime_failures;
mod sequence;
mod storage;
mod tasks;
mod variant;

use std::collections::{HashMap, HashSet};

use serde_json::{Map, Value, json};

use super::actor::{ResumeCommand, SessionActor};
use super::encode::*;
use super::protocol::{event, failure, session_error, success};
use crate::PreparedDebugTarget;
use crate::breakpoints::{BreakpointPolicy, RuntimeFailurePolicy};

/// Coarse server lifecycle visible to transport drivers.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ServerStatus {
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

/// One protocol V2 server independent from stdin/stdout ownership.
pub struct JsonlServer {
    status: ServerStatus,
    actor: SessionActor,
    execution_limits: fpas_vm::DebugExecutionLimits,
    request_ids: HashSet<u64>,
    output_cursor: usize,
    breakpoint_policies: HashMap<u64, BreakpointPolicy>,
    function_breakpoint_ids: Vec<u64>,
    runtime_failure_policy: RuntimeFailurePolicy,
    log_output_bytes: usize,
    pending_evaluation: Option<(u64, String)>,
}

impl JsonlServer {
    /// Construct a server around one prepared target.
    ///
    /// # Errors
    ///
    /// Returns a debugger initialization error for invalid runtime state.
    pub fn new(target: PreparedDebugTarget) -> Result<Self, fpas_vm::DebugSessionError> {
        let execution_limits = target.execution_limits();
        Ok(Self {
            status: ServerStatus::Created,
            actor: SessionActor::new(target.into_session()?),
            execution_limits,
            request_ids: HashSet::new(),
            output_cursor: 0,
            breakpoint_policies: HashMap::new(),
            function_breakpoint_ids: Vec::new(),
            runtime_failure_policy: RuntimeFailurePolicy::default(),
            log_output_bytes: 0,
            pending_evaluation: None,
        })
    }

    /// Return the current protocol lifecycle state.
    #[must_use]
    pub const fn status(&self) -> ServerStatus {
        self.status
    }

    /// Whether a detached call evaluation currently owns the session actor.
    #[must_use]
    pub fn is_evaluating(&self) -> bool {
        self.actor.is_evaluating()
    }

    /// Parse and handle one complete UTF-8 JSON object line.
    ///
    /// The returned records are ordered with the request response before any caused events.
    #[must_use]
    pub fn handle_line(&mut self, line: &str) -> Vec<Value> {
        if self.status == ServerStatus::Terminated {
            return vec![event(
                "protocol_error",
                error_body(
                    "invalid_state",
                    "The debugger session is terminated.",
                    "Start a new `fpas debug` process.",
                ),
            )];
        }
        let request = match serde_json::from_str::<Value>(line) {
            Ok(Value::Object(request)) => request,
            Ok(_) => return self.fatal_request("JSONL requests must be JSON objects."),
            Err(error) => return self.fatal_request(format!("Malformed JSONL request: {error}")),
        };
        let request_id = match request
            .get("id")
            .and_then(Value::as_u64)
            .filter(|id| *id > 0)
        {
            Some(id) => id,
            None => return self.fatal_request("Request field `id` must be a positive integer."),
        };
        let command = match request.get("command").and_then(Value::as_str) {
            Some(command) => command.to_string(),
            None => {
                return vec![failure(
                    request_id,
                    "<missing>",
                    "invalid_request",
                    "Request field `command` must be a string.",
                    "Send a command listed by the protocol V1 contract.",
                )];
            }
        };
        if request.get("type").and_then(Value::as_str) != Some("request") {
            return vec![failure(
                request_id,
                &command,
                "invalid_request",
                "Request field `type` must equal `request`.",
                "Use the JSONL V1 request envelope.",
            )];
        }
        if !self.request_ids.insert(request_id) {
            return vec![failure(
                request_id,
                &command,
                "invalid_request",
                format!("Request ID {request_id} was already used."),
                "Use a new positive request ID for every request.",
            )];
        }
        let arguments = match request.get("arguments") {
            None => Map::new(),
            Some(Value::Object(arguments)) => arguments.clone(),
            Some(_) => {
                return vec![failure(
                    request_id,
                    &command,
                    "invalid_request",
                    "Request field `arguments` must be an object.",
                    "Use `{}` when the command has no arguments.",
                )];
            }
        };
        let mut records = self.handle_request(request_id, &command, &arguments);
        records.extend(self.poll());
        records
    }

    /// Poll for records caused by an asynchronous continue or step operation.
    #[must_use]
    pub fn poll(&mut self) -> Vec<Value> {
        self.actor
            .poll()
            .map_or_else(Vec::new, |completion| self.complete_actor(completion))
    }

    /// Wait for the active resume operation and return its terminal or stopped events.
    #[must_use]
    pub fn wait(&mut self) -> Vec<Value> {
        self.actor
            .wait()
            .map_or_else(Vec::new, |completion| self.complete_actor(completion))
    }

    fn initialize(
        &mut self,
        request_id: u64,
        command: &str,
        arguments: &Map<String, Value>,
    ) -> Vec<Value> {
        if self.status != ServerStatus::Created {
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
        self.status = ServerStatus::Initialized;
        initialize_records(request_id, command, self.execution_limits)
    }

    fn launch(
        &mut self,
        request_id: u64,
        command: &str,
        arguments: &Map<String, Value>,
    ) -> Vec<Value> {
        if self.status != ServerStatus::Initialized {
            return vec![invalid_state(request_id, command, self.status)];
        }
        let stop_on_entry = arguments
            .get("stop_on_entry")
            .and_then(Value::as_bool)
            .unwrap_or(true);
        let mut records = vec![success(request_id, command, json!({"accepted": true}))];
        if stop_on_entry {
            self.status = ServerStatus::Stopped;
            if let Some(session) = self.actor.session() {
                records.push(stopped_event(session.last_stop()));
            }
        } else {
            self.status = ServerStatus::Running;
            if let Err(error) = self.actor.resume(ResumeCommand::Continue) {
                self.status = ServerStatus::Stopped;
                records.push(session_error(request_id, command, error));
            }
        }
        records
    }

    fn resume(&mut self, request_id: u64, command: &str, resume: ResumeCommand) -> Vec<Value> {
        if self.status != ServerStatus::Stopped {
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
                self.status = ServerStatus::Running;
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
        if self.status != ServerStatus::Running {
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
        if self.status != ServerStatus::Stopped {
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
        if self.status != ServerStatus::Stopped {
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
        if self.status != ServerStatus::Stopped {
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
            let mut records = self.wait();
            records.extend(self.disconnect_session_events());
            records.push(success(request_id, command, json!({"terminated": true})));
            self.status = ServerStatus::Terminated;
            records.push(event(
                "terminated",
                json!({"reason": "disconnect", "exit_code": 0}),
            ));
            return records;
        }
        if self.status == ServerStatus::Running {
            self.actor.pause();
            let mut records = vec![success(request_id, command, json!({"terminated": true}))];
            records.extend(self.wait());
            records.extend(self.disconnect_session_events());
            self.status = ServerStatus::Terminated;
            records.push(event(
                "terminated",
                json!({"reason": "disconnect", "exit_code": 0}),
            ));
            return records;
        }
        let mut records = vec![success(request_id, command, json!({"terminated": true}))];
        records.extend(self.disconnect_session_events());
        self.status = ServerStatus::Terminated;
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

    fn fatal_request(&mut self, message: impl Into<String>) -> Vec<Value> {
        self.status = ServerStatus::Terminated;
        vec![event(
            "protocol_error",
            error_body(
                "invalid_request",
                message,
                "Send one valid UTF-8 JSON request object per line.",
            ),
        )]
    }
}
