//! Protocol bytes stay distinct from captured debuggee output; live input is rejected.

use super::*;

use crate::vm::debug::DebuggeeChannelState;

fn output_session() -> DebugSession {
    const SOURCE: &str = r#"program TransportOutput;

uses Std.Console;

begin
  WriteLn('hello-raw')
end.
"#;
    let (program, diagnostics) = fpas_parser::parse(SOURCE);
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
fn live_input_rejects_without_mutating_workers_or_output() {
    let mut session = output_session();
    let before = fingerprint(&mut session);
    let before_stop = session.last_stop().clone();
    let error = session.push_debuggee_input("hello-raw");
    assert_eq!(error.kind, DebugErrorKind::LiveInputUnsupported);
    assert_eq!(fingerprint(&mut session), before);
    assert_eq!(session.last_stop(), &before_stop);
    assert_eq!(
        session.debuggee_channel_state(),
        DebuggeeChannelState::Connected
    );
}

#[test]
fn live_input_after_disconnect_is_invalid_state() {
    let mut session = output_session();
    session.disconnect();
    assert_eq!(
        session.push_debuggee_input("hello-raw").kind,
        DebugErrorKind::InvalidState
    );
}
