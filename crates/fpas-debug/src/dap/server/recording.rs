//! DAP custom-request mapping for the recording envelope and capture log.

use serde_json::{Value, json};

use super::DapServer;
use crate::engine::{DebugOp, ResponseBody};

impl DapServer {
    pub(super) fn describe_recording(&mut self, request_seq: u64, command: &str) -> Vec<Value> {
        self.core_request(request_seq, command, DebugOp::RecordingDescribe)
    }

    pub(super) fn start_recording(&mut self, request_seq: u64, command: &str) -> Vec<Value> {
        self.core_request(request_seq, command, DebugOp::Record)
    }
}

/// Translate one recording custom-request result into DAP naming.
pub(super) fn response_body(command: &str, body: &ResponseBody) -> Option<Value> {
    match (command, body) {
        (
            "fpas/recordingDescribe",
            ResponseBody::Recording {
                envelope,
                capturing,
                events,
                truncated,
            },
        ) => Some(json!({
            "version": envelope.version,
            "bytecodeVersion": envelope.bytecode_version,
            "program": envelope.program,
            "sources": envelope.sources,
            "capturing": capturing,
            "truncated": truncated,
            "replayable": false,
            "eventCount": events.len(),
            "eventLimit": fpas_vm::MAX_RECORDING_EVENTS,
            "events": events.iter().map(dap_event).collect::<Vec<_>>(),
        })),
        (
            "fpas/record",
            ResponseBody::RecordingStarted {
                capturing,
                truncated,
                event_count,
            },
        ) => Some(json!({
            "capturing": capturing,
            "truncated": truncated,
            "eventCount": event_count,
            "eventLimit": fpas_vm::MAX_RECORDING_EVENTS,
        })),
        _ => None,
    }
}

fn dap_event(event: &fpas_vm::DebugRecordingEvent) -> Value {
    match event {
        fpas_vm::DebugRecordingEvent::Input { text } => json!({
            "kind": "input",
            "text": text,
        }),
        fpas_vm::DebugRecordingEvent::Stop {
            task_id,
            reason,
            instruction,
        } => json!({
            "kind": "stop",
            "taskId": task_id,
            "reason": reason.as_str(),
            "instruction": instruction,
        }),
    }
}
