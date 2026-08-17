//! JSONL mapping for the recording envelope and capture log.

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
            Ok(envelope) => vec![success(
                request_id,
                command,
                envelope_body(
                    &envelope,
                    session.is_recording(),
                    session.recording_events(),
                ),
            )],
            Err(error) => vec![session_error(request_id, command, error)],
        }
    }

    pub(super) fn start_recording(&mut self, request_id: u64, command: &str) -> Vec<Value> {
        if !matches!(
            self.status,
            ServerStatus::Initialized | ServerStatus::Stopped
        ) {
            return vec![invalid_state(request_id, command, self.status)];
        }
        let Some(session) = self.actor.session_mut() else {
            return vec![invalid_state(request_id, command, self.status)];
        };
        session.start_recording();
        vec![success(
            request_id,
            command,
            json!({
                "capturing": true,
                "event_count": session.recording_events().len(),
            }),
        )]
    }
}

fn envelope_body(
    envelope: &fpas_vm::DebugRecordingEnvelope,
    capturing: bool,
    events: &[fpas_vm::DebugRecordingEvent],
) -> Value {
    json!({
        "version": envelope.version,
        "bytecode_version": envelope.bytecode_version,
        "program": envelope.program,
        "sources": envelope.sources,
        "capturing": capturing,
        "events": events.iter().map(event_body).collect::<Vec<_>>(),
    })
}

fn event_body(event: &fpas_vm::DebugRecordingEvent) -> Value {
    match event {
        fpas_vm::DebugRecordingEvent::Stop {
            task_id,
            reason,
            instruction,
        } => json!({
            "kind": "stop",
            "task_id": task_id,
            "reason": reason.as_str(),
            "instruction": instruction,
        }),
        fpas_vm::DebugRecordingEvent::Input { text } => json!({
            "kind": "input",
            "text": text,
        }),
    }
}
