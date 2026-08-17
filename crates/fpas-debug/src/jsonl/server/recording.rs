//! JSONL mapping for the recording envelope.

use serde_json::{Value, json};

use super::{JsonlServer, ServerStatus};
use crate::jsonl::encode::invalid_state;
use crate::jsonl::protocol::{session_error, success};

impl JsonlServer {
    pub(super) fn describe_recording(&mut self, request_id: u64, command: &str) -> Vec<Value> {
        if !matches!(
            self.status,
            ServerStatus::Initialized | ServerStatus::Stopped
        ) {
            return vec![invalid_state(request_id, command, self.status)];
        }
        let Some(session) = self.actor.session() else {
            return vec![invalid_state(request_id, command, self.status)];
        };
        match session.recording_envelope() {
            Ok(envelope) => vec![success(request_id, command, envelope_body(&envelope))],
            Err(error) => vec![session_error(request_id, command, error)],
        }
    }
}

fn envelope_body(envelope: &fpas_vm::DebugRecordingEnvelope) -> Value {
    json!({
        "version": envelope.version,
        "bytecode_version": envelope.bytecode_version,
        "program": envelope.program,
        "sources": envelope.sources,
    })
}
