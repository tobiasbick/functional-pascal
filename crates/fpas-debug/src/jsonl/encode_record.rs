//! Encode typed debug engine records into JSONL objects.

use serde_json::{Map, Value, json};

use super::encode::{
    breakpoint_body, data_breakpoint_body, frame_body, function_breakpoint_body, identity_body,
    initialize_body, scope_body, task_body, variable_body,
};
use super::protocol::{event, success};
use crate::engine::{DebugEvent, DebugRecord, EngineFailure, ResponseBody};

/// Encode one engine record for the JSONL adapter.
#[must_use]
pub(crate) fn encode_record(record: DebugRecord) -> Value {
    match record {
        DebugRecord::Response {
            request_id,
            command,
            outcome: Ok(body),
        } => success(request_id, command.name(), encode_body(body)),
        DebugRecord::Response {
            request_id,
            command,
            outcome: Err(error),
        } => encode_failure(request_id, command.name(), error),
        DebugRecord::Event(debug_event) => encode_event(debug_event),
    }
}

fn encode_failure(request_id: u64, command: &str, error: EngineFailure) -> Value {
    let mut error_json = json!({
        "code": error.code,
        "message": error.message,
        "help": error.help,
    });
    if let (Some(offset), Some(length), Value::Object(body)) =
        (error.offset, error.length, &mut error_json)
    {
        body.insert("offset".into(), json!(offset));
        body.insert("length".into(), json!(length));
    }
    json!({
        "type": "response",
        "request_id": request_id,
        "command": command,
        "success": false,
        "error": error_json,
    })
}

fn encode_event(debug_event: DebugEvent) -> Value {
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
                "code": format!("F{:04}", diagnostic.code.value()),
                "message": diagnostic.message,
                "help": diagnostic.help,
                "line": diagnostic.span.line(),
                "column": diagnostic.span.column(),
                "source_id": diagnostic.span.source_id(),
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

fn encode_body(body: ResponseBody) -> Value {
    match body {
        ResponseBody::Accepted => json!({"accepted": true}),
        ResponseBody::Requested => json!({"requested": true}),
        ResponseBody::Cancelled { cancelled } => json!({"cancelled": cancelled}),
        ResponseBody::TerminatedAck => json!({"terminated": true}),
        ResponseBody::Eof => json!({"eof": true}),
        ResponseBody::Cleared => json!({"cleared": true}),
        ResponseBody::Initialize {
            execution,
            hot_reload,
        } => initialize_body(execution, hot_reload),
        ResponseBody::Evaluate(result) => evaluation_json(&result),
        ResponseBody::Tasks { tasks, total } => json!({
            "tasks": tasks.iter().map(task_body).collect::<Vec<_>>(),
            "total": total
        }),
        ResponseBody::TaskHold { task_id, paused } => json!({"task_id": task_id, "paused": paused}),
        ResponseBody::TaskCancelled { task_id } => {
            json!({"task_id": task_id, "state": "cancelled"})
        }
        ResponseBody::Stack {
            frames,
            total,
            task_id,
        } => json!({
            "frames": frames.iter().map(frame_body).collect::<Vec<_>>(),
            "total": total,
            "task_id": task_id
        }),
        ResponseBody::Scopes { scopes } => json!({
            "scopes": scopes.iter().map(scope_body).collect::<Vec<_>>()
        }),
        ResponseBody::Variables { variables, total } => json!({
            "variables": variables.iter().map(variable_body).collect::<Vec<_>>(),
            "total": total
        }),
        ResponseBody::InputQueued {
            bytes,
            session_bytes,
        } => json!({"bytes": bytes, "session_bytes": session_bytes}),
        ResponseBody::Breakpoint(breakpoint) => breakpoint_body(&breakpoint),
        ResponseBody::UnverifiedBreakpoint {
            source,
            line,
            column,
            message,
            error_code,
            error_offset,
            error_length,
        } => json!({
            "verified": false,
            "message": message,
            "error_code": error_code,
            "error_offset": error_offset,
            "error_length": error_length,
            "requested": {"source": source, "line": line, "column": column}
        }),
        ResponseBody::BreakpointCleared { breakpoint_id } => {
            json!({"breakpoint_id": breakpoint_id})
        }
        ResponseBody::FunctionBreakpoints { breakpoints } => json!({
            "breakpoints": breakpoints.iter().map(function_breakpoint_body).collect::<Vec<_>>()
        }),
        ResponseBody::DataBreakpoints { breakpoints } => json!({
            "breakpoints": breakpoints.iter().map(data_breakpoint_body).collect::<Vec<_>>()
        }),
        ResponseBody::RuntimeFilters { filters } => json!({"filters": filters}),
        ResponseBody::Dictionary(result) => {
            let mut body = evaluation_map(&result.dictionary);
            if let Some(removed) = result.removed {
                body.insert("removed".into(), Value::String(removed));
            }
            if let Some(old_key) = result.old_key {
                body.insert("old_key".into(), Value::String(old_key));
            }
            if let Some(new_key) = result.new_key {
                body.insert("new_key".into(), Value::String(new_key));
            }
            Value::Object(body)
        }
        ResponseBody::Array(result) => {
            let mut body = evaluation_map(&result.array);
            body.insert("index".into(), Value::from(result.index));
            if let Some(removed) = result.removed {
                body.insert("removed".into(), Value::String(removed));
            }
            Value::Object(body)
        }
        ResponseBody::StringCharacter(result) => {
            let mut body = evaluation_map(&result.string);
            body.insert("index".into(), Value::from(result.index));
            body.insert("old_character".into(), Value::String(result.old_character));
            body.insert("new_character".into(), Value::String(result.new_character));
            Value::Object(body)
        }
        ResponseBody::ForcedReturn(result) => json!({
            "task_id": result.task_id,
            "result": result.value,
            "type_name": result.type_name,
            "variables_reference": result.variables_reference,
            "named_variables": result.named_variables,
            "indexed_variables": result.indexed_variables,
            "unwound_frames": result.unwound_frames,
            "frame": result.frame.as_ref().map(frame_body),
            "terminated": result.terminated
        }),
        ResponseBody::FrameRestart(result) => json!({
            "task_id": result.task_id,
            "frame": frame_body(&result.frame),
            "discarded_frames": result.discarded_frames
        }),
        ResponseBody::Location(location) => {
            let mut body = json!({
                "kind": location.kind.as_str(),
                "lifetime": location.lifetime.as_str(),
                "descendant": location.descendant,
            });
            if let (Value::Object(body), Some(identity)) = (&mut body, location.identity) {
                body.insert("identity".into(), identity_body(identity));
            }
            body
        }
        ResponseBody::Recording {
            envelope,
            capturing,
            events,
            truncated,
        } => json!({
            "version": envelope.version,
            "bytecode_version": envelope.bytecode_version,
            "program": envelope.program,
            "sources": envelope.sources,
            "capturing": capturing,
            "truncated": truncated,
            "replayable": false,
            "event_count": events.len(),
            "event_limit": fpas_vm::MAX_RECORDING_EVENTS,
            "events": events.iter().map(recording_event_json).collect::<Vec<_>>(),
        }),
        ResponseBody::RecordingStarted {
            capturing,
            truncated,
            event_count,
        } => json!({
            "capturing": capturing,
            "truncated": truncated,
            "event_count": event_count,
            "event_limit": fpas_vm::MAX_RECORDING_EVENTS,
        }),
        ResponseBody::LiveImage {
            class,
            accepted,
            applied,
            version,
            rollback_available,
        } => json!({
            "class": class.as_str(),
            "accepted": accepted,
            "applied": applied,
            "version": version,
            "rollback_available": rollback_available,
            "accepted_classes": fpas_vm::LiveImageUpdateClass::ACCEPTED
                .iter()
                .map(|class| class.as_str())
                .collect::<Vec<_>>(),
            "rejected_classes": fpas_vm::LiveImageUpdateClass::REJECTED
                .iter()
                .map(|class| class.as_str())
                .collect::<Vec<_>>(),
        }),
        ResponseBody::VariantDescription {
            target,
            description,
        } => json!({
            "target": target,
            "type_name": description.type_name,
            "variants": description.variants.iter().map(|variant| {
                json!({
                    "name": variant.name,
                    "fields": variant.fields.iter().map(|field| {
                        json!({
                            "name": field.name,
                            "type_name": field.type_name
                        })
                    }).collect::<Vec<_>>()
                })
            }).collect::<Vec<_>>()
        }),
        ResponseBody::VariantConstruct(result) => json!({
            "result": result.value.value,
            "type_name": result.value.type_name,
            "variables_reference": result.value.variables_reference,
            "named_variables": result.value.named_variables,
            "indexed_variables": result.value.indexed_variables,
            "variant": result.variant
        }),
        ResponseBody::Storage(result) => json!({
            "root": result.root,
            "target": result.target,
            "root_value": result.root_value,
            "value": result.value.value,
            "type": result.value.type_name,
            "variables_reference": result.value.variables_reference,
            "named_variables": result.value.named_variables,
            "indexed_variables": result.value.indexed_variables
        }),
        ResponseBody::TaskResult(result) => json!({
            "task_id": result.task_id,
            "result": result.value,
            "type_name": result.type_name,
            "variables_reference": result.variables_reference,
            "named_variables": result.named_variables,
            "indexed_variables": result.indexed_variables
        }),
    }
}

fn evaluation_json(result: &fpas_vm::DebugEvaluateResult) -> Value {
    Value::Object(evaluation_map(result))
}

fn evaluation_map(result: &fpas_vm::DebugEvaluateResult) -> Map<String, Value> {
    Map::from_iter([
        ("result".into(), Value::String(result.value.clone())),
        ("type_name".into(), Value::String(result.type_name.clone())),
        (
            "variables_reference".into(),
            Value::from(result.variables_reference),
        ),
        (
            "named_variables".into(),
            Value::from(result.named_variables),
        ),
        (
            "indexed_variables".into(),
            Value::from(result.indexed_variables),
        ),
    ])
}

fn recording_event_json(event: &fpas_vm::DebugRecordingEvent) -> Value {
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

fn location_json(location: &fpas_vm::SourceLocation) -> Value {
    json!({"source":location.source,"line":location.line,"column":location.column})
}

/// Encode an adapter-level JSONL failure that never entered the engine.
#[must_use]
pub(crate) fn encode_engine_failure(request_id: u64, command: &str, error: EngineFailure) -> Value {
    encode_failure(request_id, command, error)
}
