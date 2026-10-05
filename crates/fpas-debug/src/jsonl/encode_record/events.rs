//! Debugger JSONL event encoding.
//!
//! Documentation: `docs/pascal/tools/debugger-jsonl.md`.

use super::super::encode::{breakpoint_body, data_breakpoint_body, function_breakpoint_body};
use super::super::protocol::event;
use super::location_json;
use crate::engine::DebugEvent;
use serde_json::{Value, json};

/// Encodes a debug event with its available diagnostic/source coordinates.
pub(super) fn encode_event(debug_event: DebugEvent) -> Value {
    match debug_event {
        DebugEvent::Initialized => event("initialized", json!({})),
        DebugEvent::Stopped(stop) => event(
            "stopped",
            json!({
                "reason": stop.reason.as_str(),
                "task_id": stop.task_id,
                "all_tasks_stopped": true,
                "location": stop.location.as_ref().map(location_json),
                "instruction": stop.instruction,
                "call_depth": stop.call_depth,
                "breakpoint_id": stop.breakpoint_id,
                "breakpoint_ids": stop.breakpoint_ids
            }),
        ),
        DebugEvent::Task(change) => {
            let reason = match change.kind {
                fpas_vm::DebugTaskEventKind::Started => "started",
                fpas_vm::DebugTaskEventKind::Exited => "exited",
            };
            event("task", json!({"reason": reason, "task_id": change.task_id}))
        }
        DebugEvent::Output {
            category,
            text,
            sequence,
            breakpoint_id,
            location,
        } => {
            let mut body = json!({"category": category, "text": text});
            if let (Some(sequence), Value::Object(body)) = (sequence, &mut body) {
                body.insert("sequence".into(), json!(sequence));
            }
            if let (Some(breakpoint_id), Value::Object(body)) = (breakpoint_id, &mut body) {
                body.insert("breakpoint_id".into(), json!(breakpoint_id));
            }
            if let (Some(location), Value::Object(body)) = (location, &mut body) {
                body.insert("location".into(), location_json(&location));
            }
            event("output", body)
        }
        DebugEvent::Terminated {
            reason,
            exit_code,
            diagnostic_code,
            instruction_count,
        } => {
            let mut body = json!({"reason": reason, "exit_code": exit_code});
            if let (Some(code), Value::Object(body)) = (diagnostic_code, &mut body) {
                body.insert("diagnostic_code".into(), json!(code));
            }
            if let (Some(count), Value::Object(body)) = (instruction_count, &mut body) {
                body.insert("instruction_count".into(), json!(count));
            }
            event("terminated", body)
        }
        DebugEvent::RuntimeError {
            diagnostic,
            task_id,
        } => event(
            "runtime_error",
            json!({
                "code": diagnostic.code.to_string(),
                "message": diagnostic.message,
                "help": diagnostic.help,
                "line": diagnostic.span.map(|span| span.line()),
                "column": diagnostic.span.map(|span| span.column()),
                "source_id": diagnostic.span.map(|span| span.source_id()),
                "task_id": task_id
            }),
        ),
        DebugEvent::ProtocolError(error) => event(
            "protocol_error",
            json!({
                "code": error.code,
                "message": error.message,
                "help": error.help
            }),
        ),
        DebugEvent::SourceBreakpoint(breakpoint) => {
            event("breakpoint", breakpoint_body(&breakpoint))
        }
        DebugEvent::FunctionBreakpoint(breakpoint) => {
            event("breakpoint", function_breakpoint_body(&breakpoint))
        }
        DebugEvent::DataBreakpoint(breakpoint) => {
            event("breakpoint", data_breakpoint_body(&breakpoint))
        }
    }
}
