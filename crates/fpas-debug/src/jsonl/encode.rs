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
    let evaluation = fpas_vm::DebugEvaluationLimits::default();
    vec![
        success(
            request_id,
            command,
            json!({
                "protocol": "fpas-debug-jsonl",
                "version": 2,
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
                    "task_threads": true,
                    "evaluate": true,
                    "evaluate_calls": true,
                    "set_variable": true,
                    "conditional_breakpoints": true,
                    "hit_conditions": true,
                    "logpoints": true,
                    "reverse_execution": false
                },
                "limits": {
                    "stack_frames": inspection.max_frames,
                    "variables": inspection.max_children,
                    "value_depth": inspection.max_depth,
                    "string_characters": inspection.max_string_chars,
                    "retained_handles": inspection.max_handles,
                    "expression_bytes": evaluation.max_expression_bytes,
                    "expression_depth": evaluation.max_depth,
                    "expression_operations": evaluation.max_operations,
                    "expression_traversals": evaluation.max_traversals,
                    "expression_output_bytes": evaluation.max_output_bytes,
                    "evaluation_calls": evaluation.max_calls,
                    "evaluation_call_depth": evaluation.max_call_depth,
                    "evaluation_call_instructions": evaluation.max_call_instructions,
                    "evaluation_detached_values": evaluation.max_detached_values,
                    "evaluation_call_timeout_milliseconds": evaluation.call_timeout.as_millis(),
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
        json!({"reason":stop_reason(stop.reason),"task_id":stop.task_id,"all_tasks_stopped":true,"location":stop.location.as_ref().map(location_body),"instruction":stop.instruction,"call_depth":stop.call_depth,"breakpoint_id":stop.breakpoint_id,"breakpoint_ids":stop.breakpoint_ids}),
    )
}

/// Parses an optional non-negative integer request argument.
pub(super) fn optional_u64_argument(
    request_id: u64,
    command: &str,
    arguments: &Map<String, Value>,
    name: &str,
) -> Result<Option<u64>, Value> {
    match arguments.get(name) {
        None | Some(Value::Null) => Ok(None),
        Some(value) => value.as_u64().map(Some).ok_or_else(|| {
            failure(
                request_id,
                command,
                "invalid_request",
                format!("Command `{command}` argument `{name}` must be a non-negative integer."),
                format!("Pass a task ID returned by `tasks` as `{name}`."),
            )
        }),
    }
}

pub(super) fn task_body(task: &fpas_vm::DebugTask) -> Value {
    json!({
        "task_id": task.id,
        "name": task.name,
        "state": task.state.as_str(),
        "inspectable": task.inspectable
    })
}

pub(super) fn task_event(change: fpas_vm::DebugTaskEvent) -> Value {
    let reason = match change.kind {
        fpas_vm::DebugTaskEventKind::Started => "started",
        fpas_vm::DebugTaskEventKind::Exited => "exited",
    };
    event("task", json!({"reason": reason, "task_id": change.task_id}))
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

pub(super) fn diagnostic_body(diagnostic: &fpas_diagnostics::Diagnostic, task_id: u64) -> Value {
    json!({"code":format!("F{:04}",diagnostic.code.value()),"message":diagnostic.message,"help":diagnostic.help,"line":diagnostic.span.line(),"column":diagnostic.span.column(),"source_id":diagnostic.span.source_id(),"task_id":task_id})
}

pub(super) fn error_body(code: &str, message: impl Into<String>, help: impl Into<String>) -> Value {
    json!({"code":code,"message":message.into(),"help":help.into()})
}

pub(super) fn error_code(kind: fpas_vm::DebugErrorKind) -> &'static str {
    match kind {
        fpas_vm::DebugErrorKind::InvalidState => "invalid_state",
        fpas_vm::DebugErrorKind::UnknownTask => "unknown_task",
        fpas_vm::DebugErrorKind::UnknownBreakpoint => "unknown_breakpoint",
        fpas_vm::DebugErrorKind::UnknownFrame => "unknown_frame",
        fpas_vm::DebugErrorKind::UnknownVariablesReference => "unknown_variables_reference",
        fpas_vm::DebugErrorKind::VariableTargetUnknown => "variable_target_unknown",
        fpas_vm::DebugErrorKind::VariableTargetExpired => "variable_target_expired",
        fpas_vm::DebugErrorKind::VariableNotMutable => "variable_not_mutable",
        fpas_vm::DebugErrorKind::VariablePathUnsupported => "variable_path_unsupported",
        fpas_vm::DebugErrorKind::VariableUninitialized => "variable_uninitialized",
        fpas_vm::DebugErrorKind::VariableValueType => "variable_value_type",
        fpas_vm::DebugErrorKind::VariableUnavailable => "variable_unavailable",
        fpas_vm::DebugErrorKind::InspectionLimit => "limit_exceeded",
        fpas_vm::DebugErrorKind::UnknownName => "unknown_name",
        fpas_vm::DebugErrorKind::UninitializedValue => "uninitialized_value",
        fpas_vm::DebugErrorKind::EvaluationType => "evaluation_type",
        fpas_vm::DebugErrorKind::EvaluationDomain => "evaluation_domain",
        fpas_vm::DebugErrorKind::EvaluationLimit => "evaluation_limit",
        fpas_vm::DebugErrorKind::UnavailableValue => "unavailable_value",
        fpas_vm::DebugErrorKind::UnknownCallable => "call_target_unknown",
        fpas_vm::DebugErrorKind::AmbiguousCallable => "call_ambiguous",
        fpas_vm::DebugErrorKind::CallArity => "call_arity",
        fpas_vm::DebugErrorKind::ForbiddenCallEffect => "call_effect_forbidden",
        fpas_vm::DebugErrorKind::CallLimit => "call_limit",
        fpas_vm::DebugErrorKind::CallTimeout => "call_timeout",
        fpas_vm::DebugErrorKind::CallCancelled => "call_cancelled",
        fpas_vm::DebugErrorKind::CallRuntime => "call_runtime",
        fpas_vm::DebugErrorKind::ExecutionTimeout => "timeout",
        fpas_vm::DebugErrorKind::InstructionLimit => "instruction_limit",
        fpas_vm::DebugErrorKind::OutputLimit => "output_limit",
    }
}
