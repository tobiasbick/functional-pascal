//! JSON construction and stable debugger error mapping.

use serde_json::{Value, json};

pub(super) fn success(request_id: u64, command: &str, body: Value) -> Value {
    json!({
        "type": "response",
        "request_id": request_id,
        "command": command,
        "success": true,
        "body": body,
    })
}

pub(super) fn failure(
    request_id: u64,
    command: &str,
    code: &str,
    message: impl Into<String>,
    help: impl Into<String>,
) -> Value {
    json!({
        "type": "response",
        "request_id": request_id,
        "command": command,
        "success": false,
        "error": {
            "code": code,
            "message": message.into(),
            "help": help.into(),
        },
    })
}

pub(super) fn event(name: &str, body: Value) -> Value {
    json!({"type": "event", "event": name, "body": body})
}

pub(super) fn session_error(
    request_id: u64,
    command: &str,
    error: fpas_vm::DebugSessionError,
) -> Value {
    let code = match error.kind {
        fpas_vm::DebugErrorKind::InvalidState => "invalid_state",
        fpas_vm::DebugErrorKind::UnsupportedTasks => "tasks_unsupported",
        fpas_vm::DebugErrorKind::UnknownBreakpoint => "unknown_breakpoint",
        fpas_vm::DebugErrorKind::UnknownFrame => "unknown_frame",
        fpas_vm::DebugErrorKind::UnknownVariablesReference => "unknown_variables_reference",
        fpas_vm::DebugErrorKind::InspectionLimit => "limit_exceeded",
        fpas_vm::DebugErrorKind::ExecutionTimeout => "timeout",
        fpas_vm::DebugErrorKind::InstructionLimit => "instruction_limit",
        fpas_vm::DebugErrorKind::OutputLimit => "output_limit",
    };
    failure(request_id, command, code, error.message, error.hint)
}
