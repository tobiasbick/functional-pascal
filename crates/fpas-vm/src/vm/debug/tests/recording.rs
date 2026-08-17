//! Recording envelope identity without host paths.

use super::*;
use crate::vm::debug::{DebugRecordingEnvelope, DebugRecordingEvent, RECORDING_ENVELOPE_VERSION};

fn compile_session(source: &str) -> DebugSession {
    let (program, diagnostics) = fpas_parser::parse(source);
    assert!(diagnostics.is_empty(), "parse diagnostics: {diagnostics:?}");
    DebugSession::new(fpas_compiler::compile(&program).expect("compile recording fixture"))
        .expect("debug session")
}

fn host_path_executable(source: &str) -> VerifiedExecutable {
    Executable {
        code: vec![
            abc(Opcode::LoadUnit, 0, 0, 0),
            abc(Opcode::Return, NO_REGISTER, 0, 0),
        ],
        functions: vec![function("root", 0, 2, 1, debug(&[(0, 1)]))],
        constants: Vec::new(),
        strings: StringTable::new(vec![
            "root".to_string(),
            "helper".to_string(),
            source.to_string(),
        ]),
        globals: Vec::new(),
        records: Vec::new(),
        enums: Vec::new(),
        enum_variants: Vec::new(),
        debug_types: vec![fpas_bytecode::DebugType::Dynamic],
        source_map: SourceMap {
            sources: vec![StringId::new(2)],
            runs: vec![SourceRun {
                instruction_start: InstructionAddress::new(0),
                source: SourceId::new(0),
                line: 1,
                column: 3,
            }],
        },
        entry: FunctionId::new(0),
    }
    .verify()
    .expect("host-path fixture executable")
}

#[test]
fn envelope_names_portable_program_identity_without_host_paths() {
    let session = compile_session("program Envelope; begin end.");
    let envelope = session
        .recording_envelope()
        .expect("portable recording identity");
    assert_eq!(envelope.version, RECORDING_ENVELOPE_VERSION);
    assert_eq!(envelope.bytecode_version, fpas_bytecode::BYTECODE_VERSION);
    assert_eq!(envelope.program, "envelope");
    assert_eq!(envelope.sources, ["<memory>"]);
    assert_eq!(session.state(), DebugSessionState::Stopped);
}

#[test]
fn envelope_rejects_host_paths_without_echoing_them_or_resuming() {
    let mut session = DebugSession::new(host_path_executable("C:/secret/app.fpas"))
        .expect("session still launches");
    let error = session
        .recording_envelope()
        .expect_err("host path must be rejected");
    assert_eq!(error.kind, DebugErrorKind::RecordingHostPath);
    assert!(
        !error.message.contains("secret") && !error.hint.contains("secret"),
        "{error:?}"
    );
    assert_eq!(session.state(), DebugSessionState::Stopped);
    match session.continue_execution().expect("forward execution") {
        DebugRunResult::Terminated(_) | DebugRunResult::Stopped(_) => {}
    }
}

#[test]
fn from_executable_rejects_posix_absolute_sources() {
    let executable = host_path_executable("/abs/app.fpas");
    let error = DebugRecordingEnvelope::from_executable(&executable)
        .expect_err("posix absolute path must be rejected");
    assert_eq!(error.kind, DebugErrorKind::RecordingHostPath);
    assert!(!error.message.contains("/abs"), "{error:?}");
}

#[test]
fn recording_stays_empty_until_start() {
    let mut session = compile_session("program Quiet; begin end.");
    assert!(!session.is_recording());
    assert!(session.recording_events().is_empty());
    match session.continue_execution().expect("forward execution") {
        DebugRunResult::Terminated(_) | DebugRunResult::Stopped(_) => {}
    }
    assert!(session.recording_events().is_empty());
}

#[test]
fn start_recording_captures_current_stop_and_later_input() {
    let mut session = compile_session(
        r#"program CaptureInput;
uses Std.Console;
begin
  WriteLn(ReadLn())
end.
"#,
    );
    session.start_recording();
    assert!(session.is_recording());
    let events = session.recording_events();
    assert_eq!(events.len(), 1, "{events:?}");
    assert!(
        matches!(
            &events[0],
            DebugRecordingEvent::Stop {
                task_id: 0,
                reason: DebugStopReason::Entry,
                ..
            }
        ),
        "{events:?}"
    );
    session
        .push_debuggee_input("hello")
        .expect("queue debuggee input");
    assert!(
        session
            .recording_events()
            .iter()
            .any(|event| matches!(event, DebugRecordingEvent::Input { text } if text == "hello")),
        "{:?}",
        session.recording_events()
    );
    session.start_recording();
    assert_eq!(
        session
            .recording_events()
            .iter()
            .filter(|event| matches!(event, DebugRecordingEvent::Stop { .. }))
            .count(),
        1,
        "starting twice must not duplicate the current stop"
    );
}

#[test]
fn continue_after_record_captures_the_next_all_stop() {
    let mut session = compile_session(
        r#"program CaptureStop;
begin
  mutable var Flag: integer := 0;
  Flag := 1
end.
"#,
    );
    session
        .set_breakpoint(SourceBreakpoint {
            source: "<memory>".to_string(),
            line: 4,
            column: None,
        })
        .expect("breakpoint");
    session.start_recording();
    let stop = stopped(session.continue_execution().expect("run to breakpoint"));
    assert_eq!(stop.reason, DebugStopReason::Breakpoint);
    assert!(
        session.recording_events().iter().any(|event| matches!(
            event,
            DebugRecordingEvent::Stop {
                reason: DebugStopReason::Breakpoint,
                ..
            }
        )),
        "{:?}",
        session.recording_events()
    );
}
