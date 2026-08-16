//! Selected live-frame restart regressions.

use std::sync::Arc;

use fpas_bytecode::Value;

use super::*;

const ACTIVE_SOURCE: &str = r#"program RestartActive;

uses Std.Console;

function Branch(Value: integer): integer;
begin
  mutable var Local: integer := Value + 10;
  WriteLn('effect');
  return Local
end;

begin
  var Answer: integer := Branch(1);
  WriteLn(Answer)
end.
"#;

fn session(source: &str) -> DebugSession {
    let (program, diagnostics) = fpas_parser::parse(source);
    assert!(diagnostics.is_empty(), "parse diagnostics: {diagnostics:?}");
    DebugSession::new(fpas_compiler::compile(&program).expect("compile restart fixture"))
        .expect("debug session")
}

fn stop_in_function(session: &mut DebugSession, name: &str) -> u64 {
    for _ in 0..128 {
        let stack = session.stack(0, 16).expect("stack");
        if stack.items.first().is_some_and(|frame| frame.name == name) {
            return stack.items[0].id;
        }
        let _ = stopped(session.step_into().expect("step into function"));
    }
    panic!("{name} never became active")
}

fn frame_at_depth(session: &mut DebugSession, depth: usize) -> u64 {
    session
        .stack(0, 16)
        .expect("stack")
        .items
        .into_iter()
        .find(|frame| frame.depth == depth)
        .unwrap_or_else(|| panic!("missing frame depth {depth}"))
        .id
}

fn variable(session: &mut DebugSession, frame_id: u64, scope_name: &str, name: &str) -> String {
    let reference = session
        .scopes(frame_id)
        .expect("scopes")
        .into_iter()
        .find(|scope| scope.name == scope_name)
        .unwrap_or_else(|| panic!("missing {scope_name} scope"))
        .variables_reference;
    session
        .variables(reference, 0, 32)
        .expect("variables")
        .items
        .into_iter()
        .find(|variable| variable.name == name)
        .unwrap_or_else(|| panic!("missing {name}"))
        .value
}

fn line(source: &str, needle: &str) -> u32 {
    u32::try_from(
        source
            .lines()
            .position(|line| line.contains(needle))
            .unwrap_or_else(|| panic!("missing source marker {needle}"))
            .saturating_add(1),
    )
    .expect("line")
}

#[test]
fn active_restart_preserves_parameters_clears_locals_and_repeats_effects_only_after_continue() {
    let mut session = session(ACTIVE_SOURCE);
    let _ = stop_in_function(&mut session, "branch");
    let selected = loop {
        let frame = session.stack(0, 1).expect("stack").items[0].id;
        if variable(&mut session, frame, "Locals", "Local") == "11"
            && session.output().lines == ["effect"]
        {
            break frame;
        }
        let _ = stopped(session.step_into().expect("advance branch"));
    };
    let task_id = session.last_stop().task_id;
    let instructions = session.test_instruction_count();
    let output = session.output().lines.clone();

    let restarted = session
        .restart_frame(selected)
        .expect("restart active frame");

    assert_eq!(restarted.task_id, task_id);
    assert_eq!(restarted.frame.name, "branch");
    assert_eq!(restarted.frame.depth, 0);
    assert_eq!(restarted.discarded_frames, 0);
    assert_eq!(session.test_instruction_count(), instructions);
    assert_eq!(session.output().lines, output);
    let worker = session
        .test_worker_registers(task_id)
        .expect("restarted worker");
    assert_eq!(
        worker.4[worker.2],
        Value::Integer(1),
        "the ABI parameter prefix is retained"
    );
    assert_eq!(
        variable(&mut session, restarted.frame.id, "Locals", "Local"),
        "<uninitialized>"
    );

    let result = session
        .continue_execution()
        .expect("continue restarted frame");
    assert!(matches!(result, DebugRunResult::Terminated(_)));
    assert_eq!(session.output().lines, ["effect", "effect", "11"]);
}

#[test]
fn selected_older_restart_discards_younger_frames_and_reenters_the_selected_function() {
    const SOURCE: &str = r#"program RestartOlder;

uses Std.Console;

function Leaf(Value: integer): integer;
begin
  return Value + 1
end;

function Branch(Value: integer): integer;
begin
  var Local: integer := Value + 10;
  return Leaf(Local)
end;

begin
  WriteLn(Branch(1))
end.
"#;
    let mut session = session(SOURCE);
    let _ = stop_in_function(&mut session, "leaf");
    let branch = frame_at_depth(&mut session, 1);
    let before = session.test_instruction_count();

    let restarted = session.restart_frame(branch).expect("restart older branch");

    assert_eq!(restarted.frame.name, "branch");
    assert_eq!(restarted.discarded_frames, 1);
    assert_eq!(session.stack(0, 8).expect("restarted stack").total, 2);
    assert_eq!(session.test_instruction_count(), before);
    let worker = session.test_worker_registers(0).expect("restarted worker");
    assert_eq!(
        worker.4[worker.2],
        Value::Integer(1),
        "the selected frame ABI parameter is retained"
    );
    assert_eq!(
        variable(&mut session, restarted.frame.id, "Locals", "Local"),
        "<uninitialized>"
    );
    assert!(matches!(
        session.continue_execution().expect("continue branch"),
        DebugRunResult::Terminated(_)
    ));
    assert_eq!(session.output().lines, ["12"]);
}

#[test]
fn restart_preserves_the_exact_mutable_capture_cell() {
    const SOURCE: &str = r#"program RestartCapture;

uses Std.Console;

function Outer(Start: integer): integer;
  function Inner(): integer;
  begin
    Counter := Counter + 1;
    WriteLn(Counter);
    return Counter
  end;
begin
  mutable var Counter: integer := Start;
  return Inner()
end;

begin
  WriteLn(Outer(5))
end.
"#;
    let mut session = session(SOURCE);
    let breakpoint = session
        .set_breakpoint(SourceBreakpoint {
            source: "<memory>".to_string(),
            line: line(SOURCE, "return Counter"),
            column: None,
        })
        .expect("capture breakpoint");
    let stop = stopped(session.continue_execution().expect("run to capture return"));
    assert_eq!(stop.reason, DebugStopReason::Breakpoint);
    session
        .clear_breakpoint(breakpoint.id)
        .expect("clear capture breakpoint");
    let selected = session.stack(0, 1).expect("inner stack").items[0].id;
    let task_id = session.last_stop().task_id;
    let before = session
        .test_worker_registers(task_id)
        .expect("worker state");
    let Value::Cell(before_cell) = &before.4[before.2] else {
        panic!("inner capture must be a cell")
    };
    let before_cell = Arc::clone(before_cell);
    assert_eq!(
        *before_cell
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner()),
        Value::Integer(6)
    );

    let restarted = session.restart_frame(selected).expect("restart inner");
    let after = session
        .test_worker_registers(task_id)
        .expect("worker state");
    let Value::Cell(after_cell) = &after.4[after.2] else {
        panic!("restarted inner capture must be a cell")
    };
    assert!(Arc::ptr_eq(&before_cell, after_cell));
    assert_eq!(restarted.frame.name, "outer.inner");
    assert!(matches!(
        session.continue_execution().expect("continue inner"),
        DebugRunResult::Terminated(_)
    ));
    assert_eq!(session.output().lines, ["6", "7", "7"]);
}

#[test]
fn peer_and_stale_frames_are_rejected_without_worker_changes() {
    const SOURCE: &str = r#"program RestartPeer;

uses Std.Task;

function Work(): integer;
begin
  return 7
end;

begin
  var Pending: task := go Work();
  Wait(Pending)
end.
"#;
    let mut session = session(SOURCE);
    session
        .set_breakpoint(SourceBreakpoint {
            source: "<memory>".to_string(),
            line: line(SOURCE, "return 7"),
            column: None,
        })
        .expect("child breakpoint");
    let stop = stopped(session.continue_execution().expect("run to child"));
    assert_eq!(stop.task_id, 1);
    let root = session.stack_for_task(0, 0, 1).expect("root stack").items[0].id;
    let root_before = session.test_worker_registers(0).expect("root worker");
    let child_before = session.test_worker_registers(1).expect("child worker");
    let error = session.restart_frame(root).expect_err("peer restart");
    assert_eq!(error.kind, DebugErrorKind::FrameRestartUnsupported);
    assert_eq!(
        session.test_worker_registers(0).expect("root worker"),
        root_before
    );
    assert_eq!(
        session.test_worker_registers(1).expect("child worker"),
        child_before
    );

    let child = session.stack_for_task(1, 0, 1).expect("child stack").items[0].id;
    session.restart_frame(child).expect("restart child");
    assert_eq!(
        session.restart_frame(child).expect_err("stale frame").kind,
        DebugErrorKind::UnknownFrame
    );
}
