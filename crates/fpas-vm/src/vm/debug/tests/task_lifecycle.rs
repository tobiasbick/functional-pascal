//! Task cancel publishes waiter-visible failure without command-time dispatch.

use super::*;

use fpas_diagnostics::codes::RUNTIME_TASK_CANCELLED;

type TaskFingerprint = (u64, DebugTaskState, bool, Option<(u16, usize, usize)>);
type StateFingerprint = (u64, Vec<TaskFingerprint>);

fn fingerprint(session: &mut DebugSession) -> StateFingerprint {
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
            (task.id, task.state, task.inspectable, worker)
        })
        .collect();
    (count, tasks)
}

fn retained_session() -> DebugSession {
    const SOURCE: &str = r#"program TaskLifecycle;

uses Std.Task;

function Work(): integer;
begin
  mutable var Value: integer := 40;
  Value := Value + 2;
  return Value
end;

begin
  var Pending: task := go Work();
  Wait(Pending)
end.
"#;
    let (program, diagnostics) = fpas_parser::parse(SOURCE);
    assert!(diagnostics.is_empty(), "parse diagnostics: {diagnostics:?}");
    DebugSession::new(fpas_compiler::compile(&program).expect("compile lifecycle fixture"))
        .expect("debug session")
}

fn stop_in_child(session: &mut DebugSession) {
    session
        .set_breakpoint(SourceBreakpoint {
            source: "<memory>".to_string(),
            line: 8,
            column: None,
        })
        .expect("child breakpoint");
    let stop = stopped(session.continue_execution().expect("stop in child"));
    assert_eq!(stop.task_id, 1);
    let _ = session.take_task_events();
}

#[test]
fn cancel_rejects_unknown_completed_failed_and_root_without_mutation() {
    let mut session = retained_session();
    stop_in_child(&mut session);
    let frame = session.stack_for_task(1, 0, 8).expect("child stack").items[0].id;
    let before_stop = session.last_stop().clone();
    let before = fingerprint(&mut session);

    assert_eq!(
        session.cancel_task(99).expect_err("unknown").kind,
        DebugErrorKind::UnknownTask
    );
    assert_eq!(
        session.cancel_task(0).expect_err("root").kind,
        DebugErrorKind::InvalidState
    );
    let _ = session
        .scopes(frame)
        .expect("current stop still inspectable");
    assert_eq!(session.last_stop(), &before_stop);
    assert_eq!(fingerprint(&mut session), before);

    let mut completed_session = DebugSession::new(task_executable()).expect("task debug session");
    completed_session
        .set_breakpoint(SourceBreakpoint {
            source: "test.fpas".to_string(),
            line: 10,
            column: None,
        })
        .expect("child breakpoint");
    let child_stop = stopped(
        completed_session
            .continue_execution()
            .expect("stop in detached child"),
    );
    assert_eq!(child_stop.task_id, 1);
    let completed = stopped(completed_session.step_into_task(1).expect("complete child"));
    assert_eq!(completed.task_id, 1);
    let after_complete = fingerprint(&mut completed_session);
    assert_eq!(
        completed_session
            .cancel_task(1)
            .expect_err("completed")
            .kind,
        DebugErrorKind::UnknownTask
    );
    assert_eq!(fingerprint(&mut completed_session), after_complete);

    let mut failed = DebugSession::new(panic_executable()).expect("panic session");
    stopped(failed.step_into().expect("step to panic"));
    let stop = stopped(failed.continue_execution().expect("runtime failure"));
    assert_eq!(stop.reason, DebugStopReason::RuntimeError);
    let before_failed = failed.test_instruction_count();
    assert_eq!(
        failed.cancel_task(0).expect_err("failed session").kind,
        DebugErrorKind::InvalidState
    );
    assert_eq!(failed.test_instruction_count(), before_failed);
}

#[test]
fn cancel_child_marks_cancelled_and_stores_failure_without_dispatch() {
    let mut session = retained_session();
    stop_in_child(&mut session);
    let before_stop = session.last_stop().clone();
    let before_count = session.test_instruction_count();
    let before_main = session
        .test_worker_registers(0)
        .expect("main window before cancel");
    let before_child = session
        .test_worker_registers(1)
        .expect("child window before cancel");

    session.cancel_task(1).expect("cancel child");

    assert_eq!(session.test_instruction_count(), before_count);
    assert_eq!(session.last_stop(), &before_stop);
    assert_eq!(
        session.test_worker_registers(0).expect("main still frozen"),
        before_main
    );
    assert_eq!(
        session
            .test_worker_registers(1)
            .expect("child registers kept"),
        before_child
    );
    let child = session
        .tasks(0, 8)
        .expect("catalog")
        .items
        .into_iter()
        .find(|task| task.id == 1)
        .expect("cancelled child");
    assert_eq!(child.state, DebugTaskState::Cancelled);
    assert!(!child.inspectable);
    assert_eq!(
        session
            .stack_for_task(1, 0, 8)
            .expect_err("no fabricated stack")
            .kind,
        DebugErrorKind::UnknownTask
    );
    assert!(
        session
            .take_task_events()
            .iter()
            .any(|event| event.task_id == 1 && event.kind == crate::DebugTaskEventKind::Exited)
    );
    let crate::vm::TaskResultPoll::Failed(error) = session.test_poll_task_result(1) else {
        panic!("expected stored cancellation failure");
    };
    assert_eq!(error.code, RUNTIME_TASK_CANCELLED);
}

#[test]
fn continue_after_cancel_observes_failure_on_resume() {
    let mut session = retained_session();
    stop_in_child(&mut session);
    session.cancel_task(1).expect("cancel child");
    let _ = session.take_task_events();
    let before_child = session
        .test_worker_registers(1)
        .expect("cancelled child window");

    let stop = stopped(session.continue_execution().expect("resume waiter"));
    assert_eq!(stop.reason, DebugStopReason::RuntimeError);
    assert_eq!(stop.task_id, 0);
    assert_eq!(
        stop.diagnostic.as_ref().map(|diagnostic| diagnostic.code),
        Some(RUNTIME_TASK_CANCELLED)
    );
    assert_eq!(
        session
            .test_worker_registers(1)
            .expect("cancelled child still frozen"),
        before_child
    );
}

#[test]
fn cancel_does_not_run_paused_or_unpaused_peers() {
    let mut session = retained_session();
    stop_in_child(&mut session);
    session.pause_task(0).expect("pause main");
    let before_main = session.test_worker_registers(0).expect("paused main");
    let before_count = session.test_instruction_count();

    session.cancel_task(1).expect("cancel paused-peer child");

    assert_eq!(session.test_instruction_count(), before_count);
    assert_eq!(
        session.test_worker_registers(0).expect("main still held"),
        before_main
    );
    assert!(
        session
            .tasks(0, 8)
            .expect("catalog")
            .items
            .iter()
            .find(|task| task.id == 0)
            .expect("main")
            .paused
    );
}

#[test]
fn create_and_restart_reject_without_mutation() {
    let mut session = retained_session();
    stop_in_child(&mut session);
    let before_stop = session.last_stop().clone();
    let before = fingerprint(&mut session);

    assert_eq!(
        session.create_task().kind,
        DebugErrorKind::TaskCreateUnsupported
    );
    assert_eq!(
        session.restart_task(Some(1)).kind,
        DebugErrorKind::TaskRestartUnsupported
    );
    assert_eq!(
        session.restart_task(None).kind,
        DebugErrorKind::TaskRestartUnsupported
    );
    assert_eq!(
        session.restart_task(Some(99)).kind,
        DebugErrorKind::UnknownTask
    );
    assert_eq!(session.last_stop(), &before_stop);
    assert_eq!(fingerprint(&mut session), before);
}
