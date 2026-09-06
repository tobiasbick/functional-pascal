//! Pending operations and control precedence without scheduling-dependent readiness.

use super::{CANCELLED, TIMED_OUT, control_error};
use crate::vm::{TaskResultPoll, tasks::TaskScheduler, worker::Worker};
use fpas_bytecode::{TaskIntrinsic, Value};
use std::sync::Arc;
use std::time::Duration;

fn worker() -> Worker {
    let (program, errors) = fpas_parser::parse("program T; begin end.");
    assert!(errors.is_empty());
    let mut worker =
        Worker::new(Arc::new(fpas_compiler::compile(&program).expect("compile"))).expect("worker");
    worker.scheduler = Some(Arc::new(TaskScheduler::new()));
    worker
        .scheduler
        .as_ref()
        .expect("scheduler")
        .register_result(1);
    worker
}

#[test]
fn pending_timeout_returns_without_completing_or_consuming_task() {
    let mut worker = worker();
    let result = worker
        .controlled_wait_any(
            TaskIntrinsic::WaitAnyWithTimeout,
            &[Value::Array(vec![Value::Task(1)].into()), Value::Integer(5)],
            None,
        )
        .expect("wait");
    assert_eq!(result, Some(Some(control_error(TIMED_OUT))));
    assert!(matches!(
        worker.scheduler.as_ref().expect("scheduler").poll_result(1),
        TaskResultPoll::Pending
    ));
}

#[test]
fn cancellation_is_observed_without_any_task_completion() {
    let mut worker = worker();
    let source = worker.hosted.cancellations.create_source();
    let token = worker.hosted.cancellations.token(source).expect("token");
    let hosted = Arc::clone(&worker.hosted);
    let canceller = std::thread::spawn(move || {
        std::thread::sleep(Duration::from_millis(5));
        hosted.cancellations.cancel(source).expect("cancel");
    });
    let result = worker.controlled_wait_any(
        TaskIntrinsic::WaitAnyWithCancellation,
        &[
            Value::Array(vec![Value::Task(1)].into()),
            Value::OpaqueHandle(token),
        ],
        None,
    );
    canceller.join().expect("join");
    assert_eq!(result.expect("wait"), Some(Some(control_error(CANCELLED))));
    assert!(matches!(
        worker.scheduler.as_ref().expect("scheduler").poll_result(1),
        TaskResultPoll::Pending
    ));
}

#[test]
fn expired_budget_beats_late_success_but_zero_timeout_allows_initial_success() {
    let worker = worker();
    worker
        .scheduler
        .as_ref()
        .expect("scheduler")
        .store_result(1, Value::Integer(7));
    assert_eq!(
        worker
            .controlled_wait_result(&[1], None, true, false)
            .expect("expired"),
        Some(control_error(TIMED_OUT))
    );
    assert_eq!(
        worker
            .controlled_wait_result(&[1], None, true, true)
            .expect("initial"),
        Some(Value::result_ok(Value::Integer(0)))
    );
}

#[test]
fn invalid_identity_and_shutdown_failure_are_not_control_errors() {
    let worker = worker();
    let source = worker.hosted.cancellations.create_source();
    let token = worker.hosted.cancellations.token(source).expect("token");
    worker.hosted.cancellations.cancel(source).expect("cancel");
    assert_eq!(
        worker
            .controlled_wait_result(&[99], Some(token), true, false)
            .expect_err("identity")
            .code,
        fpas_diagnostics::codes::RUNTIME_INVALID_TASK
    );
    worker
        .scheduler
        .as_ref()
        .expect("scheduler")
        .request_cancel();
    assert_eq!(
        worker
            .controlled_wait_result(&[1], Some(token), true, false)
            .expect_err("shutdown")
            .code,
        fpas_diagnostics::codes::RUNTIME_VM_SHUTDOWN
    );
}
