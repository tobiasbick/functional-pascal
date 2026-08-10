//! Stateful JSONL request handling and asynchronous execution event delivery.

use std::collections::HashSet;

use serde_json::{Map, Value, json};

use super::actor::{Completion, ResumeCommand, SessionActor};
use super::encode::*;
use super::protocol::{event, failure, session_error, success};
use crate::PreparedDebugTarget;

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

/// One protocol V1 server independent from stdin/stdout ownership.
pub struct JsonlServer {
    status: ServerStatus,
    actor: SessionActor,
    execution_limits: fpas_vm::DebugExecutionLimits,
    request_ids: HashSet<u64>,
    output_cursor: usize,
}

impl JsonlServer {
    /// Construct a server around one prepared target.
    ///
    /// # Errors
    ///
    /// Returns a debugger initialization error, including the V1 task-spawning rejection.
    pub fn new(target: PreparedDebugTarget) -> Result<Self, fpas_vm::DebugSessionError> {
        let execution_limits = target.execution_limits();
        Ok(Self {
            status: ServerStatus::Created,
            actor: SessionActor::new(target.into_session()?),
            execution_limits,
            request_ids: HashSet::new(),
            output_cursor: 0,
        })
    }

    /// Return the current protocol lifecycle state.
    #[must_use]
    pub const fn status(&self) -> ServerStatus {
        self.status
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
            .map_or_else(Vec::new, |completion| self.complete(completion))
    }

    /// Wait for the active resume operation and return its terminal or stopped events.
    #[must_use]
    pub fn wait(&mut self) -> Vec<Value> {
        self.actor
            .wait()
            .map_or_else(Vec::new, |completion| self.complete(completion))
    }

    fn handle_request(
        &mut self,
        request_id: u64,
        command: &str,
        arguments: &Map<String, Value>,
    ) -> Vec<Value> {
        match command {
            "initialize" => self.initialize(request_id, command, arguments),
            "launch" => self.launch(request_id, command, arguments),
            "breakpoint.set" => self.set_breakpoint(request_id, command, arguments),
            "breakpoint.clear" => self.clear_breakpoint(request_id, command, arguments),
            "continue" => self.resume(request_id, command, ResumeCommand::Continue),
            "step_into" => self.resume(request_id, command, ResumeCommand::StepInto),
            "step_over" => self.resume(request_id, command, ResumeCommand::StepOver),
            "step_out" => self.resume(request_id, command, ResumeCommand::StepOut),
            "pause" => self.pause(request_id, command),
            "stack" => self.stack(request_id, command, arguments),
            "scopes" => self.scopes(request_id, command, arguments),
            "variables" => self.variables(request_id, command, arguments),
            "disconnect" => self.disconnect(request_id, command),
            _ => vec![failure(
                request_id,
                command,
                "unsupported_capability",
                format!("Debugger command `{command}` is not supported by protocol V1."),
                "Use a command advertised by `initialize`.",
            )],
        }
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
            .unwrap_or(1);
        if version != 1 {
            return vec![failure(
                request_id,
                command,
                "unsupported_protocol_version",
                format!("Protocol version {version} is unsupported."),
                "Request version 1.",
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

    fn set_breakpoint(
        &mut self,
        request_id: u64,
        command: &str,
        arguments: &Map<String, Value>,
    ) -> Vec<Value> {
        if !matches!(
            self.status,
            ServerStatus::Initialized | ServerStatus::Stopped
        ) {
            return vec![invalid_state(request_id, command, self.status)];
        }
        let Some(source) = arguments.get("source").and_then(Value::as_str) else {
            return vec![missing_argument(request_id, command, "source")];
        };
        let Some(line) = arguments
            .get("line")
            .and_then(Value::as_u64)
            .and_then(|line| u32::try_from(line).ok())
            .filter(|line| *line > 0)
        else {
            return vec![missing_argument(request_id, command, "line")];
        };
        let column = arguments
            .get("column")
            .and_then(Value::as_u64)
            .and_then(|column| u32::try_from(column).ok());
        let breakpoint = match self.actor.session_mut().map(|session| {
            session.set_breakpoint(fpas_vm::SourceBreakpoint {
                source: source.to_string(),
                line,
                column,
            })
        }) {
            Some(Ok(breakpoint)) => breakpoint,
            Some(Err(error)) => return vec![session_error(request_id, command, error)],
            None => return vec![invalid_state(request_id, command, self.status)],
        };
        let body = breakpoint_body(&breakpoint);
        vec![
            success(request_id, command, body.clone()),
            event("breakpoint", body),
        ]
    }

    fn clear_breakpoint(
        &mut self,
        request_id: u64,
        command: &str,
        arguments: &Map<String, Value>,
    ) -> Vec<Value> {
        if !matches!(
            self.status,
            ServerStatus::Initialized | ServerStatus::Stopped
        ) {
            return vec![invalid_state(request_id, command, self.status)];
        }
        let Some(id) = arguments.get("breakpoint_id").and_then(Value::as_u64) else {
            return vec![missing_argument(request_id, command, "breakpoint_id")];
        };
        match self
            .actor
            .session_mut()
            .map(|session| session.clear_breakpoint(id))
        {
            Some(Ok(())) => vec![success(request_id, command, json!({"breakpoint_id": id}))],
            Some(Err(error)) => vec![session_error(request_id, command, error)],
            None => vec![invalid_state(request_id, command, self.status)],
        }
    }

    fn resume(&mut self, request_id: u64, command: &str, resume: ResumeCommand) -> Vec<Value> {
        if self.status != ServerStatus::Stopped {
            return vec![invalid_state(request_id, command, self.status)];
        }
        match self.actor.resume(resume) {
            Ok(()) => {
                self.status = ServerStatus::Running;
                vec![success(request_id, command, json!({"accepted": true}))]
            }
            Err(error) => vec![session_error(request_id, command, error)],
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
        let Some(session) = self.actor.session() else {
            return vec![invalid_state(request_id, command, self.status)];
        };
        match session.stack(start, count) {
            Ok(frames) => vec![success(
                request_id,
                command,
                json!({
                    "frames": frames.items.iter().map(frame_body).collect::<Vec<_>>(),
                    "total": frames.total
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
        if self.status == ServerStatus::Running {
            self.actor.pause();
            let mut records = vec![success(request_id, command, json!({"terminated": true}))];
            records.extend(self.wait());
            self.status = ServerStatus::Terminated;
            records.push(event(
                "terminated",
                json!({"reason": "disconnect", "exit_code": 0}),
            ));
            return records;
        }
        if let Some(session) = self.actor.session_mut() {
            session.disconnect();
        }
        self.status = ServerStatus::Terminated;
        vec![
            success(request_id, command, json!({"terminated": true})),
            event(
                "terminated",
                json!({"reason": "disconnect", "exit_code": 0}),
            ),
        ]
    }

    fn complete(&mut self, completion: Completion) -> Vec<Value> {
        let Completion { session, result } = completion;
        let mut records = output_events(&session, &mut self.output_cursor);
        match result {
            Ok(fpas_vm::DebugRunResult::Stopped(stop)) => {
                self.status = ServerStatus::Stopped;
                if stop.reason == fpas_vm::DebugStopReason::RuntimeError
                    && let Some(diagnostic) = &stop.diagnostic
                {
                    records.push(event("runtime_error", diagnostic_body(diagnostic)));
                }
                records.push(stopped_event(&stop));
                self.actor.restore(session);
            }
            Ok(fpas_vm::DebugRunResult::Terminated(termination)) => {
                self.status = ServerStatus::Terminated;
                records.push(event(
                    "terminated",
                    json!({
                        "reason": "completed",
                        "exit_code": 0,
                        "instruction_count": termination.instruction_count
                    }),
                ));
            }
            Err(error) => {
                self.status = ServerStatus::Stopped;
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
