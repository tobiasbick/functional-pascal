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
fn readiness_beats_expired_budget_on_every_probe() {
    let worker = worker();
    worker
        .scheduler
        .as_ref()
        .expect("scheduler")
        .store_result(1, Value::Integer(7));
    assert_eq!(
        worker
            .controlled_wait_result(&[1], None, true)
            .expect("expired"),
        Some(Value::result_ok(Value::Integer(0)))
    );
}

#[test]
fn resumed_wait_observes_late_readiness_without_consuming_the_result() {
    let mut worker = worker();
    worker.debug_tasks = true;
    worker.task_clock = Some(Arc::new(crate::vm::tasks::TaskClock::manual()));
    worker.registers = vec![Value::Unit];
    worker.register_initialized = vec![false];
    let destination = fpas_bytecode::Register::new(0).unwrap();
    assert_eq!(
        worker
            .controlled_wait_any(
                TaskIntrinsic::WaitAnyWithTimeout,
                &[Value::Array(vec![Value::Task(1)].into()), Value::Integer(5)],
                Some(destination)
            )
            .unwrap(),
        Some(None)
    );
    worker.task_clock_ref().wait(Duration::from_millis(10));
    let scheduler = worker.scheduler.as_ref().unwrap().clone();
    scheduler.store_result(1, Value::Integer(42));
    assert!(worker.poll_task_suspension().unwrap());
    assert_eq!(
        worker.read(destination).unwrap(),
        &Value::result_ok(Value::Integer(0))
    );
    assert!(matches!(
        scheduler.poll_result(1),
        TaskResultPoll::Available(Value::Integer(42))
    ));
}

#[test]
fn failure_precedes_cancel_and_cancel_precedes_ready_without_consumption() {
    let worker = worker();
    let scheduler = worker.scheduler.as_ref().unwrap();
    scheduler.store_result(1, Value::Integer(42));
    let source = worker.hosted.cancellations.create_source();
    let token = worker.hosted.cancellations.token(source).unwrap();
    worker.hosted.cancellations.cancel(source).unwrap();
    assert_eq!(
        worker
            .controlled_wait_result(&[1], Some(token), true)
            .unwrap(),
        Some(control_error(CANCELLED))
    );
    assert!(matches!(
        scheduler.poll_result(1),
        TaskResultPoll::Available(Value::Integer(42))
    ));
    scheduler.register_result(2);
    let diagnostic = Box::new(fpas_diagnostics::Diagnostic::error_without_source(
        fpas_diagnostics::codes::RUNTIME_PROGRAM_PANIC,
        "original panic",
        Some("original help".into()),
    ));
    scheduler.store_failure(2, diagnostic.clone());
    assert_eq!(
        worker
            .controlled_wait_result(&[1, 2], Some(token), true)
            .unwrap_err(),
        diagnostic
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
            .controlled_wait_result(&[99], Some(token), true)
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
            .controlled_wait_result(&[1], Some(token), true)
            .expect_err("shutdown")
            .code,
        fpas_diagnostics::codes::RUNTIME_VM_SHUTDOWN
    );
}
