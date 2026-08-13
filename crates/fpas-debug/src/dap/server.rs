//! DAP request translation onto the JSONL debugger core.

mod dictionary;
mod mutation;
mod sequence;
mod tasks;

use std::collections::HashMap;

use serde_json::{Value, json};

use crate::PreparedDebugTarget;
use crate::jsonl::{JsonlServer, ServerStatus};
use tasks::ThreadMap;

/// Stateful DAP adapter for one prepared target.
pub struct DapServer {
    core: JsonlServer,
    sequence: u64,
    stop_on_entry: bool,
    breakpoints: HashMap<String, Vec<u64>>,
    source_paths: Vec<String>,
    sources: HashMap<String, String>,
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
        let source_paths = target.source_paths();
        let sources = target
            .sources()
            .iter()
            .map(|source| (source.path.clone(), source.content.clone()))
            .collect();
        Ok(Self {
            core: JsonlServer::new(target)?,
            sequence: 1,
            stop_on_entry: false,
            breakpoints: HashMap::new(),
            source_paths,
            sources,
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
        let mut response = match command {
            "initialize" => self.initialize(request_seq, &arguments),
            "launch" => { self.stop_on_entry = arguments.get("stopOnEntry").and_then(Value::as_bool).unwrap_or(false); vec![self.success(request_seq, command, json!({}))] }
            "setBreakpoints" => self.set_breakpoints(request_seq, &arguments),
            "configurationDone" => self.core_request(request_seq, command, "launch", json!({"stop_on_entry":self.stop_on_entry})),
            "threads" if self.core.status() == ServerStatus::Running => {
                let body = self.threads.active_threads();
                vec![self.success(request_seq, command, body)]
            }
            "threads" => self.core_request(request_seq, command, "tasks", json!({})),
            "stackTrace" => match self.task_id(&arguments, "threadId") {
                Ok(task_id) => self.core_request(request_seq, command, "stack", json!({"task_id":task_id,"start":arguments.get("startFrame").and_then(Value::as_u64).unwrap_or(0),"count":arguments.get("levels").and_then(Value::as_u64).unwrap_or(64)})),
                Err(message) => vec![self.failure(request_seq, command, &message)],
            },
            "scopes" => self.core_request(request_seq, command, "scopes", json!({"frame_id":arguments.get("frameId").cloned().unwrap_or(Value::Null)})),
            "variables" => self.core_request(request_seq, command, "variables", json!({"variables_reference":arguments.get("variablesReference").cloned().unwrap_or(Value::Null),"start":arguments.get("start").cloned().unwrap_or(json!(0)),"count":arguments.get("count").cloned().unwrap_or(json!(100))})),
            "evaluate" => self.evaluate(request_seq, command, &arguments),
            "setVariable" => self.set_variable(request_seq, command, &arguments),
            "setExpression" => self.set_expression(request_seq, command, &arguments),
            "fpas/dictionaryInsert" => {
                self.insert_dictionary(request_seq, command, &arguments)
            }
            "fpas/dictionaryRemove" => {
                self.remove_dictionary(request_seq, command, &arguments)
            }
            "fpas/dictionaryReplaceKey" => {
                self.replace_dictionary_key(request_seq, command, &arguments)
            }
            "fpas/arrayInsert" => self.insert_array(request_seq, command, &arguments),
            "fpas/arrayRemove" => self.remove_array(request_seq, command, &arguments),
            "fpas/stringReplaceCharacter" => {
                self.replace_string_character(request_seq, command, &arguments)
            }
            "cancel" => self.core_request(
                request_seq,
                command,
                "evaluate.cancel",
                json!({"request_id": arguments.get("requestId")}),
            ),
            "continue" if self.runtime_failed => {
                self.runtime_failed = false;
                self.core_request(request_seq, command, "disconnect", json!({}))
            }
            "continue" => self.core_request(request_seq, command, "continue", json!({})),
            "pause" => self.core_request(request_seq, command, "pause", json!({})),
            "next" => self.step_request(request_seq, command, "step_over", &arguments),
            "stepIn" => self.step_request(request_seq, command, "step_into", &arguments),
            "stepOut" => self.step_request(request_seq, command, "step_out", &arguments),
            "disconnect" => self.core_request(request_seq, command, "disconnect", json!({})),
            "source" => self.source(request_seq, command, &arguments),
            _ => vec![self.failure(request_seq, command, &format!("DAP request `{command}` is unsupported by the FPAS debugger."))],
        };
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
        self.core.status() == ServerStatus::Running
    }

    /// Whether the adapter has terminated.
    #[must_use]
    pub fn is_terminated(&self) -> bool {
        self.core.status() == ServerStatus::Terminated
    }

    fn initialize(&mut self, request_seq: u64, arguments: &Value) -> Vec<Value> {
        self.supports_invalidated_event = arguments
            .get("supportsInvalidatedEvent")
            .and_then(Value::as_bool)
            .unwrap_or(false);
        let records = self.core.handle_line(&core_request(
            request_seq,
            "initialize",
            json!({"version":2}),
        ));
        let mut output = vec![self.success(
            request_seq,
            "initialize",
            json!({
                "supportsConfigurationDoneRequest":true,"supportsDelayedStackTraceLoading":true,
                "supportsVariablePaging":true,
                "supportsTerminateRequest":false,"supportsEvaluateForHovers":true,
                "supportsConditionalBreakpoints":true,"supportsHitConditionalBreakpoints":true,
                "supportsLogPoints":true,
                "supportsCancelRequest":true,
                "supportsSetVariable":true,"supportsSetExpression":true,
                "supportsSingleThreadExecutionRequests":false,
                "supportsStepBack":false
            }),
        )];
        output.extend(self.translate_events(records));
        output
    }

    fn set_breakpoints(&mut self, request_seq: u64, arguments: &Value) -> Vec<Value> {
        let requested_source = arguments
            .pointer("/source/path")
            .and_then(Value::as_str)
            .unwrap_or("")
            .to_string();
        if requested_source.is_empty() {
            return vec![self.failure(
                request_seq,
                "setBreakpoints",
                "setBreakpoints requires source.path.",
            )];
        }
        let source = self.resolve_source_path(&requested_source);
        for id in self.breakpoints.remove(&source).unwrap_or_default() {
            let core_id = self.next_core_id();
            let _ = self.core.handle_line(&core_request(
                core_id,
                "breakpoint.clear",
                json!({"breakpoint_id":id}),
            ));
        }
        let mut ids = Vec::new();
        let mut bodies = Vec::new();
        for requested in arguments
            .get("breakpoints")
            .and_then(Value::as_array)
            .cloned()
            .unwrap_or_default()
        {
            let core_id = self.next_core_id();
            let records = self.core.handle_line(&core_request(
                core_id,
                "breakpoint.set",
                json!({
                    "source":source,
                    "line":requested.get("line"),
                    "column":requested.get("column"),
                    "condition":requested.get("condition"),
                    "hit_condition":requested.get("hitCondition"),
                    "log_message":requested.get("logMessage")
                }),
            ));
            if let Some(body) = records.first().and_then(|record| record.get("body")) {
                if let Some(id) = body.get("breakpoint_id").and_then(Value::as_u64) {
                    ids.push(id);
                }
                bodies.push(json!({"id":body.get("breakpoint_id"),"verified":body.get("verified"),"message":body.get("message"),"source":{"path":source},"line":body.pointer("/location/line").or_else(|| requested.get("line")),"column":body.pointer("/location/column").or_else(|| requested.get("column"))}));
            }
        }
        self.breakpoints.insert(source, ids);
        vec![self.success(request_seq, "setBreakpoints", json!({"breakpoints":bodies}))]
    }

    fn resolve_source_path(&self, requested: &str) -> String {
        let normalized = requested.replace('\\', "/");
        let mut matches = self
            .source_paths
            .iter()
            .filter(|source| normalized == **source || normalized.ends_with(&format!("/{source}")));
        let first = matches.next();
        if first.is_some() && matches.next().is_none() {
            return first.cloned().unwrap_or_else(|| requested.to_string());
        }
        requested.to_string()
    }

    fn source(&mut self, request_seq: u64, command: &str, arguments: &Value) -> Vec<Value> {
        let requested = arguments
            .pointer("/source/path")
            .and_then(Value::as_str)
            .unwrap_or("");
        let path = self.resolve_source_path(requested);
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
        self.core_request(
            request_seq,
            command,
            "evaluate",
            json!({
                "expression": arguments.get("expression").cloned().unwrap_or(Value::Null),
                "frame_id": arguments.get("frameId").cloned().unwrap_or(Value::Null),
                "async": true
            }),
        )
    }

    fn core_request(
        &mut self,
        request_seq: u64,
        dap_command: &str,
        command: &str,
        arguments: Value,
    ) -> Vec<Value> {
        let id = self.next_core_id();
        self.pending_core_requests
            .insert(id, (request_seq, dap_command.to_string()));
        let records = self.core.handle_line(&core_request(id, command, arguments));
        self.translate_core(records)
    }

    fn step_request(
        &mut self,
        request_seq: u64,
        command: &str,
        core_command: &str,
        arguments: &Value,
    ) -> Vec<Value> {
        match self.task_id(arguments, "threadId") {
            Ok(task_id) => self.core_request(
                request_seq,
                command,
                core_command,
                json!({"task_id": task_id}),
            ),
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

    fn translate_core(&mut self, records: Vec<Value>) -> Vec<Value> {
        let mut output = Vec::new();
        for record in records {
            if record.get("type").and_then(Value::as_str) == Some("response") {
                let Some(core_id) = record.get("request_id").and_then(Value::as_u64) else {
                    continue;
                };
                let Some((request_seq, command)) = self.pending_core_requests.remove(&core_id)
                else {
                    continue;
                };
                if record.get("success").and_then(Value::as_bool) == Some(true) {
                    let response_body = self.dap_response_body(
                        &command,
                        record.get("body").cloned().unwrap_or_else(|| json!({})),
                    );
                    output.push(self.success(request_seq, &command, response_body));
                } else {
                    let message = record
                        .pointer("/error/message")
                        .and_then(Value::as_str)
                        .unwrap_or("Debugger request failed.");
                    let code = record
                        .pointer("/error/code")
                        .and_then(Value::as_str)
                        .unwrap_or("debugger_request_failed");
                    let help = record
                        .pointer("/error/help")
                        .and_then(Value::as_str)
                        .unwrap_or("Retry the request after refreshing the current stopped state.");
                    output.push(self.structured_failure(
                        request_seq,
                        &command,
                        code,
                        message,
                        help,
                    ));
                }
            } else {
                output.extend(self.translate_events(vec![record]));
            }
        }
        output
    }

    fn translate_events(&mut self, records: Vec<Value>) -> Vec<Value> {
        let mut translated = Vec::new();
        for record in records {
            let Some(name) = record.get("event").and_then(Value::as_str) else {
                continue;
            };
            let body = record.get("body").cloned().unwrap_or_else(|| json!({}));
            match name {
                "initialized" => translated.push(self.event("initialized", json!({}))),
                "stopped" => {
                    self.runtime_failed = body.get("reason").and_then(Value::as_str) == Some("runtime_error");
                    let task_id = body.get("task_id").and_then(Value::as_u64).unwrap_or(0);
                    let thread_id = self.threads.thread_id(task_id);
                    translated.push(self.event("stopped", json!({"reason":dap_stop_reason(body.get("reason").and_then(Value::as_str)),"threadId":thread_id,"allThreadsStopped":true})));
                }
                "task" => {
                    let task_id = body.get("task_id").and_then(Value::as_u64).unwrap_or(0);
                    let thread_id = self.threads.thread_id(task_id);
                    let reason = body.get("reason").and_then(Value::as_str).unwrap_or("started");
                    if reason == "exited" {
                        self.threads.mark_exited(task_id);
                    }
                    translated.push(self.event("thread", json!({"reason":reason,"threadId":thread_id})));
                }
                "output" => translated.push(self.event("output", json!({
                    "category":body.get("category"),
                    "output":body.get("text"),
                    "source":body.pointer("/location/source").map(|path| json!({"path":path})),
                    "line":body.pointer("/location/line"),
                    "column":body.pointer("/location/column")
                }))),
                "terminated" => {
                    translated.push(self.event("exited", json!({"exitCode":body.get("exit_code").and_then(Value::as_i64).unwrap_or(0)})));
                    translated.push(self.event("terminated", json!({})));
                }
                "runtime_error" | "protocol_error" => translated.push(self.event("output", json!({"category":"stderr","output":format!("{}\n", body.get("message").and_then(Value::as_str).unwrap_or(name))}))),
                "breakpoint" => {}
                _ => {}
            }
        }
        translated
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

    fn dap_response_body(&mut self, command: &str, body: Value) -> Value {
        if command == "threads" {
            let tasks = body
                .get("tasks")
                .and_then(Value::as_array)
                .map(Vec::as_slice)
                .unwrap_or_default();
            self.threads.synchronize(tasks);
            self.threads.active_threads()
        } else {
            dap_body(command, body)
        }
    }
}

fn core_request(id: u64, command: &str, arguments: Value) -> String {
    json!({"type":"request","id":id,"command":command,"arguments":arguments}).to_string()
}

fn dap_stop_reason(reason: Option<&str>) -> &'static str {
    match reason {
        Some("entry") => "entry",
        Some("breakpoint") => "breakpoint",
        Some("pause") => "pause",
        Some("step") => "step",
        Some("runtime_error") => "exception",
        _ => "pause",
    }
}

fn dap_body(command: &str, body: Value) -> Value {
    if let Some(result) = mutation::custom_response_body(command, &body) {
        return result;
    }
    match command {
        "stackTrace" => {
            json!({"stackFrames":body.get("frames").and_then(Value::as_array).into_iter().flatten().map(|frame| json!({"id":frame.get("frame_id"),"name":frame.get("name"),"source":{"path":frame.pointer("/location/source")},"line":frame.pointer("/location/line").unwrap_or(&json!(1)),"column":frame.pointer("/location/column").unwrap_or(&json!(1))})).collect::<Vec<_>>(),"totalFrames":body.get("total")})
        }
        "scopes" => {
            json!({"scopes":body.get("scopes").and_then(Value::as_array).into_iter().flatten().map(|scope| json!({"name":scope.get("name"),"variablesReference":scope.get("variables_reference"),"namedVariables":scope.get("named_variables"),"expensive":scope.get("expensive")})).collect::<Vec<_>>() })
        }
        "variables" => {
            json!({"variables":body.get("variables").and_then(Value::as_array).into_iter().flatten().map(|variable| json!({"name":variable.get("name"),"value":variable.get("value"),"type":variable.get("type_name"),"variablesReference":variable.get("variables_reference"),"namedVariables":variable.get("named_variables"),"indexedVariables":variable.get("indexed_variables")})).collect::<Vec<_>>() })
        }
        "evaluate" => {
            json!({
                "result": body.get("result"),
                "type": body.get("type_name"),
                "variablesReference": body.get("variables_reference"),
                "namedVariables": body.get("named_variables"),
                "indexedVariables": body.get("indexed_variables")
            })
        }
        "setVariable" | "setExpression" => {
            json!({
                "value": body.get("result"),
                "type": body.get("type_name"),
                "variablesReference": body.get("variables_reference"),
                "namedVariables": body.get("named_variables"),
                "indexedVariables": body.get("indexed_variables")
            })
        }
        "continue" => json!({"allThreadsContinued":true}),
        _ => body,
    }
}
