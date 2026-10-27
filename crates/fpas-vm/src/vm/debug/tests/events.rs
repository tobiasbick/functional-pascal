//! TUI events remain queued while debugger inspection is stopped.

use super::*;
use crate::vm::debug::{DebugTerminalEvent, DebugTerminalKeyEvent, DebugTerminalKeyKind};
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

uses Std.Console as Console;

begin
  Console.WriteLn('before');
  if Console.EventPending() then
    Console.WriteLn('got-event'); end if;
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

uses Std.Console as Console;

begin
  if Console.EventPending() then
    Console.WriteLn('got-event'); end if;
end.
"#,
    );
    let _ = session.continue_execution().expect("resume");
    assert!(session.output().lines.is_empty());
}

#[test]
fn timed_console_input_waits_for_its_deadline_without_blocking_peer_tasks() {
    let mut session = compile_session(
        r#"program TimedConsoleInput;
uses Std.Console as Console, Std.Time as Time;
procedure ReadLater();
begin
  const Started := Time.MonotonicMillis();
  const Event := Console.ReadEventTimeout(50);
  Console.WriteLn((Event is None) and (Time.ElapsedMillis(Started) >= 50));
end procedure;
begin
  scope
    const Reader: task := go ReadLater();
    Time.Sleep(5);
    Console.WriteLn('peer');
  end scope;
end.
"#,
    );
    assert!(matches!(
        session.continue_execution().expect("timed console input"),
        DebugRunResult::Terminated(_)
    ));
    assert_eq!(session.output().lines, ["peer", "true"]);
}

#[test]
fn timed_console_input_receives_an_event_queued_during_the_wait() {
    let mut session = compile_session(
        r#"program ArrivingConsoleInput;
uses Std.Console as Console;
begin
  Console.WriteLn('waiting');
  Console.WriteLn(Console.ReadEventTimeout(5000) is Some(_));
  Console.WriteLn(Console.ReadEventTimeout(0) is None);
end.
"#,
    );
    let terminal = session.terminal_handle();
    let input = std::thread::spawn(move || {
        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(2);
        while terminal.output_byte_count() == 0 {
            assert!(std::time::Instant::now() < deadline, "reader did not start");
            std::thread::sleep(std::time::Duration::from_millis(1));
        }
        std::thread::sleep(std::time::Duration::from_millis(20));
        terminal
            .push_events(&[DebugTerminalEvent::Key(DebugTerminalKeyEvent {
                kind: DebugTerminalKeyKind::Escape,
                shift: false,
                ctrl: false,
                alt: false,
                meta: false,
            })])
            .expect("queued terminal input");
    });
    let result = session.continue_execution();
    input.join().expect("input sender");
    assert!(matches!(
        result.expect("terminal input"),
        DebugRunResult::Terminated(_)
    ));
    assert_eq!(session.output().lines, ["waiting", "true", "true"]);
}

#[test]
fn timed_console_input_is_cancelled_when_its_scope_returns() {
    let mut session = compile_session(
        r#"program CancelConsoleInput;
uses Std.Console as Console, Std.Time as Time;
procedure ReadLater();
begin
  discard Console.ReadEventTimeout(60000);
  panic('reader completed instead of being cancelled');
end procedure;
function Run(): integer;
begin
  scope
    discard go ReadLater();
    Time.Sleep(5);
    return 7;
  end scope;
end function;
begin
  Console.WriteLn(Run());
end.
"#,
    );
    assert!(matches!(
        session.continue_execution().expect("cancel console input"),
        DebugRunResult::Terminated(_)
    ));
    assert_eq!(session.output().lines, ["7"]);
}

#[test]
fn input_arrival_wakes_each_waiting_reader_after_a_debugger_stop() {
    let source = r#"program TwoReaders;
uses Std.Console as Console, Std.Time as Time;
procedure ReadLater();
begin
  Console.WriteLn(Console.ReadEventTimeout(5000) is Some(_));
end procedure;
begin
  scope
    discard go ReadLater();
    discard go ReadLater();
    Time.Sleep(5);
    Console.WriteLn('checkpoint');
  end scope;
end.
"#;
    let mut session = compile_session(source);
    let line = source
        .lines()
        .position(|line| line.contains("'checkpoint'"))
        .expect("checkpoint");
    session
        .set_breakpoint(SourceBreakpoint {
            source: "<memory>".into(),
            line: u32::try_from(line + 1).expect("source line"),
            column: None,
        })
        .expect("checkpoint breakpoint");
    assert!(matches!(
        session.continue_execution().expect("checkpoint"),
        DebugRunResult::Stopped(_)
    ));
    assert_eq!(
        session
            .tasks(0, 8)
            .expect("tasks")
            .items
            .iter()
            .filter(|task| task.state == DebugTaskState::Sleeping)
            .count(),
        2
    );
    session.test_push_console_event(ConsoleEvent::key(escape_key()));
    session.test_push_console_event(ConsoleEvent::key(escape_key()));
    assert!(matches!(
        session.continue_execution().expect("readers"),
        DebugRunResult::Terminated(_)
    ));
    assert_eq!(
        session
            .output()
            .lines
            .iter()
            .filter(|line| *line == "true")
            .count(),
        2
    );
    assert!(!session.output().lines.iter().any(|line| line == "false"));
}
