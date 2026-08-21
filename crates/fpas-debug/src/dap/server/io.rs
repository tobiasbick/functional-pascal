//! DAP mapping for queued debuggee input, EOF, and cancel.

use serde_json::{Value, json};

use super::DapServer;
use super::args;
use crate::engine::{DebugOp, ResponseBody};

pub(super) fn response_body(command: &str, body: &ResponseBody) -> Option<Value> {
    match (command, body) {
        (
            "fpas/input",
            ResponseBody::InputQueued {
                bytes,
                session_bytes,
            },
        ) => Some(json!({
            "bytes": bytes,
            "sessionBytes": session_bytes
        })),
        ("fpas/eof", ResponseBody::Eof) => Some(json!({"eof": true})),
        ("fpas/cancelInput", ResponseBody::Cleared) => Some(json!({"cleared": true})),
        _ => None,
    }
}

impl DapServer {
    pub(super) fn push_debuggee_input(
        &mut self,
        request_seq: u64,
        command: &str,
        arguments: &Value,
    ) -> Vec<Value> {
        match args::required_string(arguments, "text") {
            Ok(text) => self.core_request(request_seq, command, DebugOp::IoInput { text }),
            Err(message) => vec![self.failure(request_seq, command, &message)],
        }
    }

    pub(super) fn signal_debuggee_eof(&mut self, request_seq: u64, command: &str) -> Vec<Value> {
        self.core_request(request_seq, command, DebugOp::IoEof)
    }

    pub(super) fn cancel_debuggee_input(&mut self, request_seq: u64, command: &str) -> Vec<Value> {
        self.core_request(request_seq, command, DebugOp::IoCancel)
    }
}
