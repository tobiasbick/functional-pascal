//! Record constructors used by debug engine handlers.

use super::DebugStatus;
use super::command::DebugCommand;
use super::error::EngineFailure;
use super::record::{DebugEvent, DebugRecord, ResponseBody};
use crate::evaluation::EvaluationParseError;

pub(super) fn ok(request_id: u64, command: &str, body: ResponseBody) -> DebugRecord {
    DebugRecord::ok(request_id, DebugCommand::from_name(command), body)
}

pub(super) fn fail(request_id: u64, command: &str, error: EngineFailure) -> DebugRecord {
    DebugRecord::fail(request_id, DebugCommand::from_name(command), error)
}

pub(super) fn invalid_state(request_id: u64, command: &str, status: DebugStatus) -> DebugRecord {
    fail(
        request_id,
        command,
        EngineFailure::invalid_state(command, status),
    )
}

pub(super) fn session_error(
    request_id: u64,
    command: &str,
    error: fpas_vm::DebugSessionError,
) -> DebugRecord {
    fail(request_id, command, EngineFailure::from_session(error))
}

pub(super) fn parse_error(
    request_id: u64,
    command: &str,
    error: EvaluationParseError,
) -> DebugRecord {
    fail(request_id, command, EngineFailure::from_parse(error))
}

pub(super) fn invalid_request(
    request_id: u64,
    command: &str,
    message: impl Into<String>,
    help: impl Into<String>,
) -> DebugRecord {
    fail(
        request_id,
        command,
        EngineFailure::new("invalid_request", message, help),
    )
}

pub(super) fn unsupported(
    request_id: u64,
    command: &str,
    message: impl Into<String>,
    help: impl Into<String>,
) -> DebugRecord {
    fail(
        request_id,
        command,
        EngineFailure::new("unsupported_capability", message, help),
    )
}

pub(super) fn event(event: DebugEvent) -> DebugRecord {
    DebugRecord::event(event)
}

pub(super) fn output_events(
    session: &fpas_vm::DebugSession,
    cursor: &mut usize,
) -> Vec<DebugRecord> {
    let output = session.output();
    let records = output
        .lines
        .iter()
        .skip(*cursor)
        .enumerate()
        .map(|(index, line)| {
            event(DebugEvent::Output {
                category: "stdout",
                text: format!("{line}\n"),
                sequence: Some(cursor.saturating_add(index).saturating_add(1)),
                breakpoint_id: None,
                location: None,
            })
        })
        .collect();
    *cursor = output.lines.len();
    records
}

pub(super) fn task_event(change: fpas_vm::DebugTaskEvent) -> DebugRecord {
    event(DebugEvent::Task(change))
}
