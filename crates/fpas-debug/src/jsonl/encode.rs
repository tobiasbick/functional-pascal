//! JSONL response and event body encoding.

use serde_json::{Map, Value, json};

use super::protocol::{event, failure, success};
use super::server::ServerStatus;

pub(super) fn initialize_records(
    request_id: u64,
    command: &str,
    execution: fpas_vm::DebugExecutionLimits,
) -> Vec<Value> {
    let inspection = fpas_vm::DebugInspectionLimits::default();
    vec![
        success(
            request_id,
            command,
            json!({
                "protocol": "fpas-debug-jsonl",
                "version": 1,
                "capabilities": {
                    "source_breakpoints": true,
                    "pause": true,
                    "continue": true,
                    "step_into": true,
                    "step_over": true,
                    "step_out": true,
                    "stack_pagination": true,
                    "scope_inspection": true,
                    "variable_pagination": true,
                    "aggregate_expansion": true,
                    "structured_output": true,
                    "attach": false,
                    "task_threads": false,
                    "evaluate": false,
                    "set_variable": false,
                    "conditional_breakpoints": false,
                    "logpoints": false,
                    "reverse_execution": false
                },
                "limits": {
                    "stack_frames": inspection.max_frames,
                    "variables": inspection.max_children,
                    "value_depth": inspection.max_depth,
                    "string_characters": inspection.max_string_chars,
                    "retained_handles": inspection.max_handles,
                    "captured_output_bytes": execution.max_output_bytes,
                    "instructions": execution.max_instructions,
                    "timeout_milliseconds": execution.timeout.as_millis()
                }
            }),
        ),
        event("initialized", json!({})),
    ]
}

pub(super) fn invalid_state(request_id: u64, command: &str, state: ServerStatus) -> Value {
    failure(
        request_id,
        command,
        "invalid_state",
        format!("Command `{command}` is invalid while the protocol is {state:?}."),
        "Wait for the required lifecycle event before retrying.",
    )
}

pub(super) fn missing_argument(request_id: u64, command: &str, argument: &str) -> Value {
    failure(
        request_id,
        command,
        "invalid_request",
        format!("Command `{command}` requires argument `{argument}`."),
        "Add the required field to the request `arguments` object.",
    )
}

pub(super) fn index_argument(arguments: &Map<String, Value>, name: &str, default: usize) -> usize {
    arguments
        .get(name)
        .and_then(Value::as_u64)
        .and_then(|value| usize::try_from(value).ok())
        .unwrap_or(default)
}

pub(super) fn breakpoint_body(breakpoint: &fpas_vm::BoundBreakpoint) -> Value {
    json!({"breakpoint_id":breakpoint.id,"verified":breakpoint.is_verified(),"requested":{"source":breakpoint.requested.source,"line":breakpoint.requested.line,"column":breakpoint.requested.column},"location":breakpoint.location.as_ref().map(location_body),"message":(!breakpoint.is_verified()).then_some("No executable sequence point exists on the requested line.")})
}

pub(super) fn stopped_event(stop: &fpas_vm::DebugStop) -> Value {
    event(
        "stopped",
        json!({"reason":stop_reason(stop.reason),"thread_id":1,"location":stop.location.as_ref().map(location_body),"instruction":stop.instruction,"call_depth":stop.call_depth,"breakpoint_id":stop.breakpoint_id}),
    )
}

fn stop_reason(reason: fpas_vm::DebugStopReason) -> &'static str {
    match reason {
        fpas_vm::DebugStopReason::Entry => "entry",
        fpas_vm::DebugStopReason::Breakpoint => "breakpoint",
        fpas_vm::DebugStopReason::Pause => "pause",
        fpas_vm::DebugStopReason::Step => "step",
        fpas_vm::DebugStopReason::RuntimeError => "runtime_error",
    }
}

fn location_body(location: &fpas_vm::SourceLocation) -> Value {
    json!({"source":location.source,"line":location.line,"column":location.column})
}

pub(super) fn frame_body(frame: &fpas_vm::DebugFrame) -> Value {
    json!({"frame_id":frame.id,"name":frame.name,"location":frame.location.as_ref().map(location_body),"depth":frame.depth})
}

pub(super) fn scope_body(scope: &fpas_vm::DebugScope) -> Value {
    json!({"name":scope.name,"kind":format!("{:?}",scope.kind).to_ascii_lowercase(),"variables_reference":scope.variables_reference,"named_variables":scope.named_variables,"expensive":scope.expensive})
}

pub(super) fn variable_body(variable: &fpas_vm::DebugVariable) -> Value {
    json!({"name":variable.name,"value":variable.value,"type_name":variable.type_name,"variables_reference":variable.variables_reference,"named_variables":variable.named_variables,"indexed_variables":variable.indexed_variables,"presentation_hint":variable.presentation_hint})
}

pub(super) fn output_events(session: &fpas_vm::DebugSession, cursor: &mut usize) -> Vec<Value> {
    let output = session.output();
    let records=output.lines.iter().skip(*cursor).enumerate().map(|(index,line)| event("output",json!({"category":"stdout","text":format!("{line}\n"),"sequence":cursor.saturating_add(index).saturating_add(1)}))).collect();
    *cursor = output.lines.len();
    records
}

pub(super) fn diagnostic_body(diagnostic: &fpas_diagnostics::Diagnostic) -> Value {
    json!({"code":format!("F{:04}",diagnostic.code.value()),"message":diagnostic.message,"help":diagnostic.help,"line":diagnostic.span.line(),"column":diagnostic.span.column(),"source_id":diagnostic.span.source_id()})
}

pub(super) fn error_body(code: &str, message: impl Into<String>, help: impl Into<String>) -> Value {
    json!({"code":code,"message":message.into(),"help":help.into()})
}

pub(super) fn error_code(kind: fpas_vm::DebugErrorKind) -> &'static str {
    match kind {
        fpas_vm::DebugErrorKind::InvalidState => "invalid_state",
        fpas_vm::DebugErrorKind::UnsupportedTasks => "tasks_unsupported",
        fpas_vm::DebugErrorKind::UnknownBreakpoint => "unknown_breakpoint",
        fpas_vm::DebugErrorKind::UnknownFrame => "unknown_frame",
        fpas_vm::DebugErrorKind::UnknownVariablesReference => "unknown_variables_reference",
        fpas_vm::DebugErrorKind::InspectionLimit => "limit_exceeded",
        fpas_vm::DebugErrorKind::ExecutionTimeout => "timeout",
        fpas_vm::DebugErrorKind::InstructionLimit => "instruction_limit",
        fpas_vm::DebugErrorKind::OutputLimit => "output_limit",
    }
}
