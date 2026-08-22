//! TUI events remain queued while debugger inspection is stopped.

use super::*;
use fpas_std::{ConsoleEvent, ConsoleKeyEvent, key_kind_index};

fn compile_session(source: &str) -> DebugSession {
    let (program, diagnostics) = fpas_parser::parse(source);
    assert!(diagnostics.is_empty(), "parse diagnostics: {diagnostics:?}");
    DebugSession::new(fpas_compiler::compile(&program).expect("compile event fixture"))
        .expect("debug session")
}

fn escape_key() -> ConsoleKeyEvent {
    ConsoleKeyEvent::new(
        key_kind_index("Escape"),
        '\u{1b}',
        false,
        false,
        false,
        false,
    )
}

#[test]
fn queued_tui_events_wait_until_resume() {
    let mut session = compile_session(
        r#"program EventOwnership;

uses Std.Console;

begin
  WriteLn('before');
  if EventPending() then
    WriteLn('got-event')
end.
"#,
    );
    session.test_push_console_event(ConsoleEvent::key(escape_key()));
    let _ = session
        .evaluate(&DebugExpression::Integer(1), None)
        .expect("evaluate");
    let _ = session.stack(0, 8).expect("stack");
    assert!(session.output().lines.is_empty());
    let _ = session.continue_execution().expect("resume");
    assert_eq!(
        session.output().lines,
        vec!["before".to_string(), "got-event".to_string()]
    );
}

#[test]
fn empty_tui_queue_does_not_poll_the_terminal() {
    let mut session = compile_session(
        r#"program EventEmpty;

uses Std.Console;

begin
  if EventPending() then
    WriteLn('got-event')
end.
"#,
    );
    let _ = session.continue_execution().expect("resume");
    assert!(session.output().lines.is_empty());
}
