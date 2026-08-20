//! JSONL parsing and encoding around the protocol-neutral debugger engine.

use serde_json::{Map, Value};

use crate::engine::{DebugCommand, DebugEngine, DebugRecord, DebugRequest};
use crate::target::PreparedDebugTarget;

pub use crate::engine::DebugStatus as ServerStatus;

/// JSONL adapter for one protocol-neutral debugger engine.
pub struct JsonlServer {
    engine: DebugEngine,
}

impl JsonlServer {
    /// Construct a JSONL adapter around one prepared target.
    ///
    /// # Errors
    ///
    /// Returns debugger initialization failures for invalid runtime state.
    pub fn new(target: PreparedDebugTarget) -> Result<Self, fpas_vm::DebugSessionError> {
        Ok(Self {
            engine: DebugEngine::new(target)?,
        })
    }

    /// Return the current debugger lifecycle state.
    #[must_use]
    pub const fn status(&self) -> ServerStatus {
        self.engine.status()
    }

    pub(super) const fn terminated_fatally(&self) -> bool {
        self.engine.terminated_fatally()
    }

    /// Parse and handle one complete UTF-8 JSON object line.
    #[must_use]
    pub fn handle_line(&mut self, line: &str) -> Vec<Value> {
        let request = match serde_json::from_str::<Value>(line) {
            Ok(Value::Object(request)) => request,
            Ok(_) => return self.fatal("JSONL requests must be JSON objects."),
            Err(error) => return self.fatal(format!("Malformed JSONL request: {error}")),
        };
        let request_id = match request
            .get("id")
            .and_then(Value::as_u64)
            .filter(|id| *id > 0)
        {
            Some(id) => id,
            None => return self.fatal("Request field `id` must be a positive integer."),
        };
        let Some(command) = request.get("command").and_then(Value::as_str) else {
            return vec![crate::jsonl::protocol::failure(
                request_id,
                "<missing>",
                "invalid_request",
                "Request field `command` must be a string.",
                "Send a command listed by the protocol V1 contract.",
            )];
        };
        if request.get("type").and_then(Value::as_str) != Some("request") {
            return vec![crate::jsonl::protocol::failure(
                request_id,
                command,
                "invalid_request",
                "Request field `type` must equal `request`.",
                "Use the JSONL V1 request envelope.",
            )];
        }
        let arguments = match request.get("arguments") {
            None => Map::new(),
            Some(Value::Object(arguments)) => arguments.clone(),
            Some(_) => {
                return vec![crate::jsonl::protocol::failure(
                    request_id,
                    command,
                    "invalid_request",
                    "Request field `arguments` must be an object.",
                    "Use `{}` when the command has no arguments.",
                )];
            }
        };
        let records = self.engine.execute(DebugRequest {
            id: request_id,
            command: DebugCommand::from_name(command),
            arguments,
        });
        self.records_from_engine(records)
    }

    /// Poll the engine for asynchronous records.
    #[must_use]
    pub fn poll(&mut self) -> Vec<Value> {
        let records = self.engine.poll();
        self.records_from_engine(records)
    }

    /// Wait for the active engine operation to stop or finish.
    #[must_use]
    pub fn wait(&mut self) -> Vec<Value> {
        let records = self.engine.wait();
        self.records_from_engine(records)
    }

    /// Replace the live image through the shared debugger engine.
    ///
    /// # Errors
    ///
    /// Returns a session error when the current stopped target cannot accept
    /// the candidate image.
    pub fn replace_live_image(
        &mut self,
        candidate: &fpas_bytecode::VerifiedExecutable,
    ) -> Result<fpas_vm::LiveImageReplaceResult, fpas_vm::DebugSessionError> {
        self.engine.replace_live_image(candidate)
    }

    fn fatal(&mut self, message: impl Into<String>) -> Vec<Value> {
        let records = self.engine.fatal_protocol_error(message);
        self.records_from_engine(records)
    }

    fn records_from_engine(&self, records: Vec<DebugRecord>) -> Vec<Value> {
        records.into_iter().map(DebugRecord::into_jsonl).collect()
    }
}
