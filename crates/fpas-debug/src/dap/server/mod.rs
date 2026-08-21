//! DAP adapter: wire spelling onto the typed debug engine.

mod args;
mod breakpoints;
mod completed_result;
mod data_breakpoints;
mod dictionary;
mod dispatch;
mod events;
mod exceptions;
mod forced_return;
mod frame_restart;
mod instruction;
mod io;
mod lifecycle;
mod live_image;
mod location;
mod mutation;
mod recording;
mod response;
mod sequence;
mod source_paths;
mod storage;
mod task_control;
mod tasks;
mod values;
mod variant;

use std::collections::HashMap;

use serde_json::{Value, json};

use crate::PreparedDebugTarget;
use crate::engine::{DebugEngine, DebugOp, DebugRecord, DebugRequest, DebugStatus, ResponseBody};
use source_paths::SourcePaths;
use tasks::ThreadMap;

/// Stateful DAP adapter for one prepared target.
pub struct DapServer {
    core: DebugEngine,
    sequence: u64,
    stop_on_entry: bool,
    source_breakpoints: HashMap<String, Vec<u64>>,
    source_paths: SourcePaths,
    sources: HashMap<String, String>,
    source_revision: u64,
    runtime_failed: bool,
    pending_core_requests: HashMap<u64, (u64, String)>,
    supports_invalidated_event: bool,
    threads: ThreadMap,
}

impl DapServer {
    /// Construct an adapter around one verified target.
    ///
    /// # Errors
    ///
    /// Returns debugger initialization failures for invalid runtime state.
    pub fn new(target: PreparedDebugTarget) -> Result<Self, fpas_vm::DebugSessionError> {
        let source_paths = SourcePaths::new(&target.source_paths(), target.sources());
        let sources = target
            .sources()
            .iter()
            .map(|source| (source.path.clone(), source.content.clone()))
            .collect();
        Ok(Self {
            core: DebugEngine::new(target)?,
            sequence: 1,
            stop_on_entry: false,
            source_breakpoints: HashMap::new(),
            source_paths,
            sources,
            source_revision: 1,
            runtime_failed: false,
            pending_core_requests: HashMap::new(),
            supports_invalidated_event: false,
            threads: ThreadMap::new(),
        })
    }

    /// Handle one decoded DAP request and return ordered responses and events.
    #[must_use]
    pub fn handle(&mut self, request: Value) -> Vec<Value> {
        let Some(object) = request.as_object() else {
            return vec![self.event("output", json!({"category":"stderr","output":"Malformed DAP request: expected an object.\n"}))];
        };
        let request_seq = object.get("seq").and_then(Value::as_u64).unwrap_or(0);
        let command = object
            .get("command")
            .and_then(Value::as_str)
            .unwrap_or("<missing>");
        if object.get("type").and_then(Value::as_str) != Some("request") || request_seq == 0 {
            return vec![self.failure(
                request_seq,
                command,
                "DAP request requires type=request and a positive seq.",
            )];
        }
        let arguments = object
            .get("arguments")
            .cloned()
            .unwrap_or_else(|| json!({}));
        let mut output = if self.core.is_evaluating() && !matches!(command, "cancel" | "disconnect")
        {
            self.wait()
        } else {
            Vec::new()
        };
        let mut response = self.dispatch_request(request_seq, command, &arguments);
        output.append(&mut response);
        output
    }

    /// Wait for an active resume and translate resulting events.
    #[must_use]
    pub fn wait(&mut self) -> Vec<Value> {
        let records = self.core.wait();
        self.translate_core(records)
    }

    /// Poll an active resume without blocking.
    #[must_use]
    pub fn poll(&mut self) -> Vec<Value> {
        let records = self.core.poll();
        self.translate_core(records)
    }

    /// Whether execution is active.
    #[must_use]
    pub fn is_running(&self) -> bool {
        self.core.status() == DebugStatus::Running
    }

    /// Whether the adapter has terminated.
    #[must_use]
    pub fn is_terminated(&self) -> bool {
        self.core.status() == DebugStatus::Terminated
    }

    fn initialize(&mut self, request_seq: u64, arguments: &Value) -> Vec<Value> {
        self.supports_invalidated_event = arguments
            .get("supportsInvalidatedEvent")
            .and_then(Value::as_bool)
            .unwrap_or(false);
        let records = self
            .core
            .execute(DebugRequest::new(request_seq, DebugOp::Initialize));
        let mut output = vec![self.success(
            request_seq,
            "initialize",
            json!({
                "supportsConfigurationDoneRequest":true,"supportsDelayedStackTraceLoading":true,
                "supportsVariablePaging":true,
                "supportsTerminateRequest":false,"supportsEvaluateForHovers":true,
                "supportsConditionalBreakpoints":true,"supportsHitConditionalBreakpoints":true,
                "supportsLogPoints":true,
                "supportsFunctionBreakpoints":true,
                "exceptionBreakpointFilters":exceptions::advertised_filters(),
                "supportsCancelRequest":true,
                "supportsSetVariable":true,"supportsSetExpression":true,
                "supportsRestartFrame":true,
                "supportsGotoTargetsRequest":false,
                "supportsAttach":false,
                "supportsDataBreakpoints":true,
                "supportsDisassembleRequest":false,
                "supportsReadMemoryRequest":false,
                "supportsWriteMemoryRequest":false,
                "supportsSingleThreadExecutionRequests":false,
                "supportsStepBack":false,
                "supportsHotReload":self.core.supports_hot_reload()
            }),
        )];
        if !self.core.supports_hot_reload()
            && let Some(capabilities) = output
                .first_mut()
                .and_then(|response| response.get_mut("body"))
                .and_then(Value::as_object_mut)
        {
            capabilities.remove("supportsHotReload");
        }
        output.extend(self.translate_events(records));
        output
    }

    fn source(&mut self, request_seq: u64, command: &str, arguments: &Value) -> Vec<Value> {
        let requested = arguments
            .pointer("/source/path")
            .and_then(Value::as_str)
            .unwrap_or("");
        let path = match self.resolve_source_path(requested) {
            Ok(path) => path,
            Err(message) => return vec![self.failure(request_seq, command, message)],
        };
        match self.sources.get(&path).cloned() {
            Some(content) => vec![self.success(
                request_seq,
                command,
                json!({"content":content,"mimeType":"text/x-pascal"}),
            )],
            None => vec![self.failure(
                request_seq,
                command,
                "Verified source content is unavailable for this path. Open the workspace file directly or rebuild the target with source identities.",
            )],
        }
    }

    fn evaluate(&mut self, request_seq: u64, command: &str, arguments: &Value) -> Vec<Value> {
        let context = arguments
            .get("context")
            .and_then(Value::as_str)
            .unwrap_or("repl");
        if !matches!(context, "watch" | "repl" | "hover" | "variables") {
            return vec![self.failure(
                request_seq,
                command,
                &format!("DAP evaluate context `{context}` is unsupported; use watch, repl, hover, or variables."),
            )];
        }
        match args::required_string(arguments, "expression") {
            Ok(expression) => self.core_request(
                request_seq,
                command,
                DebugOp::Evaluate {
                    expression,
                    frame_id: arguments.get("frameId").and_then(Value::as_u64),
                    async_eval: true,
                },
            ),
            Err(message) => vec![self.failure(request_seq, command, &message)],
        }
    }

    pub(super) fn core_request(
        &mut self,
        request_seq: u64,
        dap_command: &str,
        op: DebugOp,
    ) -> Vec<Value> {
        let id = self.next_core_id();
        self.pending_core_requests
            .insert(id, (request_seq, dap_command.to_string()));
        let records = self.core.execute(DebugRequest::new(id, op));
        self.sync_sources();
        self.translate_core(records)
    }

    fn sync_sources(&mut self) {
        let revision = self.core.source_revision();
        if revision == self.source_revision {
            return;
        }
        self.source_paths = SourcePaths::new(
            &self
                .core
                .sources()
                .iter()
                .map(|source| source.path.clone())
                .collect::<Vec<_>>(),
            self.core.sources(),
        );
        self.sources = self
            .core
            .sources()
            .iter()
            .map(|source| (source.path.clone(), source.content.clone()))
            .collect();
        self.source_revision = revision;
    }

    fn step_request(
        &mut self,
        request_seq: u64,
        command: &str,
        arguments: &Value,
        op: impl FnOnce(u64) -> DebugOp,
    ) -> Vec<Value> {
        match self.task_id(arguments, "threadId") {
            Ok(task_id) => self.core_request(request_seq, command, op(task_id)),
            Err(message) => vec![self.failure(request_seq, command, &message)],
        }
    }

    fn task_id(&self, arguments: &Value, field: &str) -> Result<u64, String> {
        let thread_id = arguments
            .get(field)
            .and_then(Value::as_u64)
            .ok_or_else(|| format!("DAP request requires a known `{field}`."))?;
        self.threads.task_id(thread_id).ok_or_else(|| {
            format!(
                "DAP thread {thread_id} is unknown or expired; request `threads` and select a current FPAS task."
            )
        })
    }

    fn translate_core(&mut self, records: Vec<DebugRecord>) -> Vec<Value> {
        let mut output = Vec::new();
        for record in records {
            match record {
                DebugRecord::Response {
                    request_id,
                    outcome: Ok(body),
                    ..
                } => {
                    let Some((request_seq, command)) =
                        self.pending_core_requests.remove(&request_id)
                    else {
                        continue;
                    };
                    let response_body = self.dap_response_body(&command, body);
                    output.push(self.success(request_seq, &command, response_body));
                }
                DebugRecord::Response {
                    request_id,
                    outcome: Err(error),
                    ..
                } => {
                    let Some((request_seq, command)) =
                        self.pending_core_requests.remove(&request_id)
                    else {
                        continue;
                    };
                    output.push(self.structured_failure(
                        request_seq,
                        &command,
                        &error.code,
                        &error.message,
                        &error.help,
                    ));
                }
                DebugRecord::Event(event) => output.extend(self.translate_event(event)),
            }
        }
        output
    }

    fn success(&mut self, request_seq: u64, command: &str, body: Value) -> Value {
        let seq = self.take_seq();
        json!({"seq":seq,"type":"response","request_seq":request_seq,"success":true,"command":command,"body":body})
    }
    fn failure(&mut self, request_seq: u64, command: &str, message: &str) -> Value {
        let seq = self.take_seq();
        json!({"seq":seq,"type":"response","request_seq":request_seq,"success":false,"command":command,"message":message,"body":{"error":{"format":message,"showUser":true}}})
    }
    fn structured_failure(
        &mut self,
        request_seq: u64,
        command: &str,
        code: &str,
        message: &str,
        help: &str,
    ) -> Value {
        let seq = self.take_seq();
        json!({"seq":seq,"type":"response","request_seq":request_seq,"success":false,"command":command,"message":message,"body":{"error":{"code":code,"format":message,"help":help,"showUser":true}}})
    }
    fn event(&mut self, name: &str, body: Value) -> Value {
        let seq = self.take_seq();
        json!({"seq":seq,"type":"event","event":name,"body":body})
    }
    fn take_seq(&mut self) -> u64 {
        let seq = self.sequence;
        self.sequence = self.sequence.saturating_add(1);
        seq
    }
    fn next_core_id(&mut self) -> u64 {
        1_000_000_000u64.saturating_add(self.take_seq())
    }

    fn dap_response_body(&mut self, command: &str, body: ResponseBody) -> Value {
        if command == "threads"
            && let ResponseBody::Tasks { tasks, .. } = &body
        {
            self.threads.synchronize(tasks);
            return self.threads.active_threads();
        }
        response::dap_body(command, body)
    }
}
