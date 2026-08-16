//! Per-task pause and resume: paused peers must not execute.

use super::*;

use fpas_bytecode::FunctionId;

fn fingerprint(
    session: &mut DebugSession,
) -> (
    u64,
    Vec<(u64, DebugTaskState, bool, Option<(u16, usize, usize)>)>,
) {
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
            (task.id, task.state, task.paused, worker)
        })
        .collect();
    (count, tasks)
}

fn shared_state_session() -> DebugSession {
    const SOURCE: &str = r#"program SharedStateTaskControl;

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

fn stop_in_child(session: &mut DebugSession) {
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
}

#[test]
fn pause_task_rejects_unknown_id_without_mutation() {
    let mut session = DebugSession::new(task_executable()).expect("task debug session");
    stop_in_child(&mut session);
    let frame = session.stack_for_task(1, 0, 8).expect("child stack").items[0].id;
    let before_stop = session.last_stop().clone();
    let before = fingerprint(&mut session);

    let error = session.pause_task(99).expect_err("unknown task");
    assert_eq!(error.kind, DebugErrorKind::UnknownTask);
    let _ = session
        .scopes(frame)
        .expect("current stop still inspectable");
    let after = fingerprint(&mut session);

    assert_eq!(session.last_stop(), &before_stop);
    assert_eq!(after, before);
}

#[test]
fn pause_and_resume_reject_completed_child_without_mutation() {
    let mut session = DebugSession::new(task_executable()).expect("task debug session");
    stop_in_child(&mut session);
    let completed = stopped(session.step_into_task(1).expect("complete child"));
    assert_eq!(completed.task_id, 1);
    let before = fingerprint(&mut session);

    assert_eq!(
        session.pause_task(1).expect_err("completed pause").kind,
        DebugErrorKind::UnknownTask
    );
    assert_eq!(
        session.resume_task(1).expect_err("completed resume").kind,
        DebugErrorKind::UnknownTask
    );
    assert_eq!(fingerprint(&mut session), before);
}

#[test]
fn pause_task_rejects_a_failed_session_without_dispatch() {
    let mut session = DebugSession::new(panic_executable()).expect("panic session");
    stopped(session.step_into().expect("step to panic"));
    let stop = stopped(session.continue_execution().expect("runtime failure"));
    assert_eq!(stop.reason, DebugStopReason::RuntimeError);
    let before = session.test_instruction_count();

    assert_eq!(
        session.pause_task(0).expect_err("failed session").kind,
        DebugErrorKind::InvalidState
    );
    assert_eq!(session.test_instruction_count(), before);
}

#[test]
fn stepping_a_paused_task_rejects_without_dispatch() {
    let mut session = DebugSession::new(task_executable()).expect("task debug session");
    stop_in_child(&mut session);
    session.pause_task(1).expect("pause child");
    let before_stop = session.last_stop().clone();
    let before = fingerprint(&mut session);

    assert_eq!(
        session
            .step_into_task(1)
            .expect_err("step paused child")
            .kind,
        DebugErrorKind::InvalidState
    );
    assert_eq!(session.last_stop(), &before_stop);
    assert_eq!(fingerprint(&mut session), before);
}

#[test]
fn paused_peer_does_not_run_as_wait_dependency() {
    let mut session =
        DebugSession::with_manual_clock(task_state_executable()).expect("waiting-step session");
    session
        .set_breakpoint(SourceBreakpoint {
            source: "test.fpas".to_string(),
            line: 2,
            column: None,
        })
        .expect("main wait breakpoint");
    let main = stopped(session.continue_execution().expect("main wait stop"));
    assert_eq!(
        (main.task_id, main.location.map(|location| location.line)),
        (0, Some(2))
    );
    session.pause_task(2).expect("pause stopper dependency");
    session
        .set_breakpoint(SourceBreakpoint {
            source: "test.fpas".to_string(),
            line: 21,
            column: None,
        })
        .expect("dependency breakpoint");
    let before_stopper = session
        .test_worker_registers(2)
        .expect("paused stopper window");

    let blocked = stopped(
        session
            .step_into_task(0)
            .expect("step waiter without running paused stopper"),
    );
    assert_ne!(blocked.task_id, 2);
    assert_ne!(blocked.reason, DebugStopReason::Breakpoint);
    assert_eq!(
        session
            .test_worker_registers(2)
            .expect("stopper still frozen"),
        before_stopper
    );
    assert!(
        session
            .tasks(0, 8)
            .expect("catalog")
            .items
            .iter()
            .find(|task| task.id == 2)
            .expect("stopper")
            .paused
    );
}

#[test]
fn pause_then_resume_then_continue_runs_the_held_task() {
    let mut session = shared_state_session();
    let bound = session
        .replace_function_breakpoints(vec![FunctionBreakpoint {
            name: "Writer".to_string(),
        }])
        .expect("writer breakpoint");
    assert!(bound[0].is_verified());
    let stop = stopped(session.continue_execution().expect("stop in writer"));
    assert_eq!(stop.task_id, 1);
    session.pause_task(1).expect("pause writer");
    session.pause_task(1).expect("idempotent pause");
    assert!(
        session
            .tasks(0, 8)
            .expect("catalog")
            .items
            .iter()
            .find(|task| task.id == 1)
            .expect("writer")
            .paused
    );

    session.resume_task(1).expect("resume writer");
    assert!(
        !session
            .tasks(0, 8)
            .expect("catalog")
            .items
            .iter()
            .find(|task| task.id == 1)
            .expect("writer")
            .paused
    );

    let frozen_count = session.test_instruction_count();
    let mut stored = session
        .evaluate(&DebugExpression::Name("Shared".to_string()), None)
        .expect("shared before resume")
        .value;
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
fn continue_skips_a_paused_child_until_resume() {
    let mut session = DebugSession::new(task_executable()).expect("task debug session");
    stop_in_child(&mut session);
    session.pause_task(1).expect("pause child");
    let before_child = session
        .test_worker_registers(1)
        .expect("paused child window");

    let blocked = stopped(
        session
            .continue_execution()
            .expect("continue without child"),
    );
    assert_eq!(blocked.reason, DebugStopReason::Pause);
    assert_eq!(
        session.test_worker_registers(1).expect("still frozen"),
        before_child
    );
    assert!(
        session
            .tasks(0, 8)
            .expect("catalog")
            .items
            .iter()
            .find(|task| task.id == 1)
            .expect("child")
            .paused
    );

    session.resume_task(1).expect("resume child");
    let completed = stopped(session.step_into_task(1).expect("run resumed child"));
    assert_eq!(completed.task_id, 1);
    assert_eq!(
        session
            .stack_for_task(1, 0, 8)
            .expect_err("completed child")
            .kind,
        DebugErrorKind::UnknownTask
    );
}

#[test]
fn newly_admitted_tasks_start_unpaused() {
    let mut session = DebugSession::new(pending_child_executable()).expect("pending-child session");
    session.test_enqueue_pending_task(FunctionId::new(1));
    session.pause_task(0).expect("pause main");
    session
        .set_breakpoint(SourceBreakpoint {
            source: "test.fpas".to_string(),
            line: 10,
            column: None,
        })
        .expect("child breakpoint");
    let stop = stopped(session.continue_execution().expect("admit unpaused child"));
    assert_eq!(stop.task_id, 1);
    let catalog = session.tasks(0, 8).expect("admitted catalog");
    let child = catalog
        .items
        .iter()
        .find(|task| task.id == 1)
        .expect("child");
    assert!(!child.paused);
    let main = catalog
        .items
        .iter()
        .find(|task| task.id == 0)
        .expect("main");
    assert!(main.paused);
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
