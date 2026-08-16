//! Protocol bytes stay distinct from captured debuggee output; queued input is ordered.

use super::*;

use crate::vm::debug::DebuggeeChannelState;

fn output_session() -> DebugSession {
    compile_session(
        r#"program TransportOutput;

uses Std.Console;

begin
  WriteLn('hello-raw')
end.
"#,
    )
}

fn readln_session() -> DebugSession {
    compile_session(
        r#"program TransportInput;

uses Std.Console;

begin
  WriteLn(ReadLn());
  WriteLn(ReadLn())
end.
"#,
    )
}

fn compile_session(source: &str) -> DebugSession {
    let (program, diagnostics) = fpas_parser::parse(source);
    assert!(diagnostics.is_empty(), "parse diagnostics: {diagnostics:?}");
    DebugSession::new(fpas_compiler::compile(&program).expect("compile transport fixture"))
        .expect("debug session")
}

fn fingerprint(session: &mut DebugSession) -> (u64, DebugSessionState, Vec<String>, u32) {
    (
        session.test_instruction_count(),
        session.state(),
        session.output().lines,
        session.last_stop().instruction,
    )
}

#[test]
fn captured_output_stays_on_the_session_channel() {
    let mut session = output_session();
    assert_eq!(
        session.debuggee_channel_state(),
        DebuggeeChannelState::Connected
    );
    assert!(session.output().lines.is_empty());
    let _ = session.continue_execution().expect("run to completion");
    assert_eq!(session.output().lines, vec!["hello-raw".to_string()]);
}

#[test]
fn disconnect_closes_the_channel_without_running_remaining_output() {
    let mut session = output_session();
    assert_eq!(session.last_stop().reason, DebugStopReason::Entry);
    assert!(session.output().lines.is_empty());
    session.disconnect();
    assert_eq!(session.state(), DebugSessionState::Terminated);
    assert_eq!(
        session.debuggee_channel_state(),
        DebuggeeChannelState::Closed
    );
    assert!(session.output().lines.is_empty());
    assert_eq!(
        session
            .continue_execution()
            .expect_err("terminated session")
            .kind,
        DebugErrorKind::InvalidState
    );
}

#[test]
fn queued_lines_are_consumed_in_order() {
    let mut session = readln_session();
    session.push_debuggee_input("one").expect("first line");
    session.push_debuggee_input("two").expect("second line");
    let _ = session.continue_execution().expect("consume queued input");
    assert_eq!(
        session.output().lines,
        vec!["one".to_string(), "two".to_string()]
    );
}

#[test]
fn missing_input_fails_without_reading_protocol_stdin() {
    let mut session = readln_session();
    let stop = stopped(session.continue_execution().expect("missing input"));
    assert_eq!(stop.reason, DebugStopReason::RuntimeError);
    let diagnostic = stop.diagnostic.expect("missing-input diagnostic");
    assert!(diagnostic.message.contains("no input available"));
    assert!(session.output().lines.is_empty());
}

#[test]
fn eof_is_a_first_class_input_event() {
    let mut session = readln_session();
    session.signal_debuggee_eof().expect("eof");
    let stop = stopped(session.continue_execution().expect("eof continue"));
    assert_eq!(stop.reason, DebugStopReason::RuntimeError);
    let diagnostic = stop.diagnostic.expect("eof diagnostic");
    assert!(diagnostic.message.contains("end of input"));
}

#[test]
fn input_after_eof_rejects_without_mutation() {
    let mut session = readln_session();
    session.signal_debuggee_eof().expect("eof");
    let before = fingerprint(&mut session);
    let before_stop = session.last_stop().clone();
    let error = session.push_debuggee_input("late").expect_err("closed");
    assert_eq!(error.kind, DebugErrorKind::DebuggeeInputClosed);
    assert_eq!(fingerprint(&mut session), before);
    assert_eq!(session.last_stop(), &before_stop);
}

#[test]
fn input_limit_rejects_without_mutation() {
    let mut session = DebugSession::with_limits(
        {
            let source = r#"program TransportLimit;

uses Std.Console;

begin
  WriteLn(ReadLn())
end.
"#;
            let (program, diagnostics) = fpas_parser::parse(source);
            assert!(diagnostics.is_empty(), "parse diagnostics: {diagnostics:?}");
            fpas_compiler::compile(&program).expect("compile limit fixture")
        },
        Vec::new(),
        DebugInspectionLimits::default(),
        DebugExecutionLimits {
            max_input_bytes: 1,
            ..DebugExecutionLimits::default()
        },
    )
    .expect("limited session");
    let before = fingerprint(&mut session);
    let error = session.push_debuggee_input("ab").expect_err("limit");
    assert_eq!(error.kind, DebugErrorKind::DebuggeeInputLimit);
    assert_eq!(fingerprint(&mut session), before);
}

#[test]
fn cancel_drops_unread_input() {
    let mut session = readln_session();
    session.push_debuggee_input("secret").expect("queued");
    session.cancel_debuggee_input().expect("cleared");
    let stop = stopped(session.continue_execution().expect("cancelled input"));
    assert_eq!(stop.reason, DebugStopReason::RuntimeError);
    assert!(session.output().lines.is_empty());
}

#[test]
fn cancel_does_not_reset_the_session_input_quota() {
    let mut session = readln_session();
    let first = session.push_debuggee_input("a").expect("queued");
    session.cancel_debuggee_input().expect("cleared");
    let second = session.push_debuggee_input("b").expect("quota retained");
    assert_eq!(first.bytes, 2);
    assert_eq!(second.session_bytes, 4);
}

#[test]
fn live_input_after_disconnect_is_invalid_state() {
    let mut session = output_session();
    session.disconnect();
    assert_eq!(
        session
            .push_debuggee_input("hello-raw")
            .expect_err("terminated")
            .kind,
        DebugErrorKind::InvalidState
    );
}
