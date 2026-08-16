//! All-stop quiescence: inspection must not dispatch, admit, or wake tasks.

use super::*;

use std::time::Duration;

fn fingerprint(
    session: &mut DebugSession,
) -> (u64, Vec<(u64, DebugTaskState, Option<(u16, usize, usize)>)>) {
    let count = session.test_instruction_count();
    let tasks = session
        .tasks(0, 16)
        .expect("catalog")
        .items
        .into_iter()
        .map(|task| {
            let worker = session
                .test_worker_registers(task.id)
                .map(|snapshot| (snapshot.0, snapshot.1, snapshot.3));
            (task.id, task.state, worker)
        })
        .collect();
    (count, tasks)
}

fn pending_child_executable() -> VerifiedExecutable {
    executable(
        vec![
            abc(Opcode::LoadUnit, 0, 0, 0),
            Instruction::abx(Opcode::Jump, 0, 0).expect("root loop"),
            abc(Opcode::LoadUnit, 0, 0, 0),
            abc(Opcode::Return, NO_REGISTER, 0, 0),
        ],
        vec![
            function("root", 0, 2, 1, debug(&[(0, 1)])),
            function("helper", 2, 4, 1, debug(&[(2, 10)])),
        ],
        Vec::new(),
        vec![(0, 1), (2, 10)],
    )
}

fn shared_state_session() -> DebugSession {
    const SOURCE: &str = r#"program SharedStateQuiescence;

uses Std.Task;

mutable var Shared: integer := 0;

procedure Writer();
begin
  Shared := 1
end;

begin
  var Pending: task := go Writer();
  Wait(Pending)
end.
"#;
    let (program, diagnostics) = fpas_parser::parse(SOURCE);
    assert!(diagnostics.is_empty(), "parse diagnostics: {diagnostics:?}");
    DebugSession::new(fpas_compiler::compile(&program).expect("compile shared-state fixture"))
        .expect("debug session")
}

#[test]
fn stopped_inspection_does_not_dispatch_or_admit_tasks() {
    let mut session = DebugSession::new(task_executable()).expect("task debug session");
    session
        .set_breakpoint(SourceBreakpoint {
            source: "test.fpas".to_string(),
            line: 10,
            column: None,
        })
        .expect("child breakpoint");
    let stop = stopped(session.continue_execution().expect("stop in child"));
    assert_eq!(stop.task_id, 1);
    let _ = session.take_task_events();

    let before_stop = session.last_stop().clone();
    let before = fingerprint(&mut session);
    let _ = session.stack_for_task(0, 0, 8).expect("main stack");
    let child = session.stack_for_task(1, 0, 8).expect("child stack");
    assert!(!child.items.is_empty());
    let after = fingerprint(&mut session);

    assert_eq!(session.state(), DebugSessionState::Stopped);
    assert_eq!(session.last_stop(), &before_stop);
    assert_eq!(after, before);
    assert!(session.take_task_events().is_empty());
}

#[test]
fn frozen_waiting_and_sleeping_peers_keep_their_instruction_windows() {
    let mut session =
        DebugSession::with_manual_clock(task_state_executable()).expect("task-state debug session");
    session
        .set_breakpoint(SourceBreakpoint {
            source: "test.fpas".to_string(),
            line: 21,
            column: None,
        })
        .expect("stopper breakpoint");
    let stop = stopped(session.continue_execution().expect("task-state stop"));
    assert_eq!(stop.task_id, 2);
    let _ = session.take_task_events();

    let before = fingerprint(&mut session);
    let waiting = session.stack_for_task(0, 0, 8).expect("waiting main");
    let sleeping = session.stack_for_task(1, 0, 8).expect("sleeping peer");
    assert!(!waiting.items.is_empty());
    assert!(!sleeping.items.is_empty());
    let after = fingerprint(&mut session);

    assert_eq!(after, before);
    assert_eq!(
        after
            .1
            .iter()
            .map(|item| (item.0, item.1))
            .collect::<Vec<_>>(),
        vec![
            (0, DebugTaskState::Waiting),
            (1, DebugTaskState::Sleeping),
            (2, DebugTaskState::Runnable),
        ]
    );
}

#[test]
fn catalog_does_not_admit_queued_spawns_until_resume() {
    let mut session = DebugSession::new(pending_child_executable()).expect("pending-child session");
    session.test_enqueue_pending_task(FunctionId::new(1));
    let before = fingerprint(&mut session);
    assert_eq!(
        before.1.iter().map(|item| item.0).collect::<Vec<_>>(),
        vec![0]
    );
    assert!(session.take_task_events().is_empty());

    session
        .set_breakpoint(SourceBreakpoint {
            source: "test.fpas".to_string(),
            line: 10,
            column: None,
        })
        .expect("child breakpoint");
    let stop = stopped(session.continue_execution().expect("admit on resume"));
    assert_eq!(stop.task_id, 1);
    assert_eq!(stop.reason, DebugStopReason::Breakpoint);
    let tasks = session.tasks(0, 8).expect("admitted catalog");
    assert!(tasks.items.iter().any(|task| task.id == 1));
}

#[test]
fn advancing_the_clock_during_a_stop_does_not_wake_sleeping_peers() {
    let mut session =
        DebugSession::with_manual_clock(task_state_executable()).expect("task-state debug session");
    session
        .set_breakpoint(SourceBreakpoint {
            source: "test.fpas".to_string(),
            line: 21,
            column: None,
        })
        .expect("stopper breakpoint");
    let _ = stopped(session.continue_execution().expect("task-state stop"));
    let before = fingerprint(&mut session);

    session.test_advance_clock(Duration::from_millis(1_000));
    let _ = session.stack_for_task(1, 0, 8).expect("sleeping stack");
    let after = fingerprint(&mut session);

    assert_eq!(after, before);
    assert_eq!(
        after.1.iter().find(|item| item.0 == 1).map(|item| item.1),
        Some(DebugTaskState::Sleeping)
    );
}

#[test]
fn shared_globals_are_stable_while_writer_and_waiter_are_stopped() {
    let mut session = shared_state_session();
    let bound = session
        .replace_function_breakpoints(vec![FunctionBreakpoint {
            name: "Writer".to_string(),
        }])
        .expect("writer breakpoint");
    assert!(bound[0].is_verified());

    let stop = stopped(session.continue_execution().expect("stop in writer"));
    assert_eq!(stop.task_id, 1);
    let before = fingerprint(&mut session);
    let child_frame = session.stack_for_task(1, 0, 8).expect("writer stack").items[0].id;
    let from_writer = session
        .evaluate(
            &DebugExpression::Name("Shared".to_string()),
            Some(child_frame),
        )
        .expect("writer global");
    let from_root = session
        .evaluate(&DebugExpression::Name("Shared".to_string()), None)
        .expect("root global");
    session.select_task(0).expect("inspect waiter");
    let waiter_frame = session.stack(0, 8).expect("waiter stack").items[0].id;
    let from_waiter = session
        .evaluate(
            &DebugExpression::Name("Shared".to_string()),
            Some(waiter_frame),
        )
        .expect("waiter global");
    let after = fingerprint(&mut session);

    assert_eq!(from_writer.value, "0");
    assert_eq!(from_root.value, "0");
    assert_eq!(from_waiter.value, "0");
    assert_eq!(after, before);

    let frozen_count = session.test_instruction_count();
    let mut stored = from_root.value.clone();
    for _ in 0..32 {
        if stored == "1" {
            break;
        }
        match session.step_into_task(1) {
            Ok(result) => {
                let _ = stopped(result);
            }
            Err(_) => break,
        }
        stored = session
            .evaluate(&DebugExpression::Name("Shared".to_string()), None)
            .expect("shared after resume")
            .value;
    }
    assert_eq!(stored, "1");
    assert!(session.test_instruction_count() > frozen_count);
}

#[test]
fn failed_inspection_does_not_dispatch_and_resume_invalidates_handles() {
    let mut session = DebugSession::new(panic_executable()).expect("panic session");
    stopped(session.step_into().expect("step to panic"));
    let stop = stopped(session.continue_execution().expect("runtime failure"));
    assert_eq!(stop.reason, DebugStopReason::RuntimeError);
    assert_eq!(session.state(), DebugSessionState::Failed);
    let frame = session.stack(0, 8).expect("failed stack").items[0].id;
    let before = session.test_instruction_count();
    let _ = session.tasks(0, 8).expect("failed catalog");
    let _ = session.scopes(frame).expect("failed scopes");
    assert_eq!(session.test_instruction_count(), before);
    assert_eq!(
        session
            .continue_execution()
            .expect_err("failed session")
            .kind,
        DebugErrorKind::InvalidState
    );

    let mut live = DebugSession::new(task_executable()).expect("live session");
    live.set_breakpoint(SourceBreakpoint {
        source: "test.fpas".to_string(),
        line: 10,
        column: None,
    })
    .expect("child breakpoint");
    let _ = stopped(live.continue_execution().expect("child stop"));
    let stale = live.stack_for_task(1, 0, 8).expect("child stack").items[0].id;
    let _ = stopped(live.step_into_task(1).expect("resume"));
    assert_eq!(
        live.scopes(stale).expect_err("expired generation").kind,
        DebugErrorKind::UnknownFrame
    );
}

#[test]
fn completed_child_has_no_fabricated_stack() {
    let mut session = DebugSession::new(task_executable()).expect("task debug session");
    session
        .set_breakpoint(SourceBreakpoint {
            source: "test.fpas".to_string(),
            line: 10,
            column: None,
        })
        .expect("child breakpoint");
    let _ = stopped(session.continue_execution().expect("stop in child"));
    let completed = stopped(session.step_into_task(1).expect("complete child"));
    assert_eq!(completed.task_id, 1);
    assert_eq!(
        session
            .stack_for_task(1, 0, 8)
            .expect_err("completed child")
            .kind,
        DebugErrorKind::UnknownTask
    );
    let catalog = session.tasks(0, 8).expect("terminal catalog");
    let child = catalog
        .items
        .iter()
        .find(|task| task.id == 1)
        .expect("completed child remains visible");
    assert_eq!(child.state, DebugTaskState::Completed);
    assert!(!child.inspectable);
}
