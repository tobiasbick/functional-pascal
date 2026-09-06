//! Non-consuming completion selection and wakeup regressions.

use super::TaskScheduler;
use crate::vm::{TaskAnyPoll, TaskResultPoll};
use fpas_bytecode::Value;
use std::sync::{Arc, mpsc};
use std::time::Duration;

#[test]
fn selection_preserves_input_order_duplicates_and_results() {
    let scheduler = TaskScheduler::new();
    for id in [1, 2, 3] {
        scheduler.register_result(id);
    }
    assert!(matches!(
        scheduler.poll_any(&[3, 1, 2]),
        TaskAnyPoll::Pending
    ));
    scheduler.store_result(2, Value::Integer(22));
    assert!(matches!(
        scheduler.poll_any(&[3, 1, 2]),
        TaskAnyPoll::Complete(2)
    ));
    scheduler.store_result(3, Value::Integer(33));
    assert!(matches!(
        scheduler.poll_any(&[3, 2, 3]),
        TaskAnyPoll::Complete(0)
    ));
    assert!(matches!(
        scheduler.poll_result(3),
        TaskResultPoll::Available(Value::Integer(33))
    ));
    assert!(matches!(
        scheduler.poll_any(&[3, 2]),
        TaskAnyPoll::Complete(0)
    ));
    assert!(matches!(scheduler.poll_result(3), TaskResultPoll::Consumed));
    assert!(matches!(
        scheduler.poll_result(2),
        TaskResultPoll::Available(Value::Integer(22))
    ));
}

#[test]
fn invalid_identity_precedes_failure_which_precedes_success() {
    let scheduler = TaskScheduler::new();
    for id in [1, 2, 3] {
        scheduler.register_result(id);
    }
    scheduler.store_result(1, Value::Unit);
    let error = super::tests::failure("original");
    scheduler.store_failure(2, error.clone());
    scheduler.store_failure(3, super::tests::failure("later"));
    assert!(matches!(
        scheduler.poll_any(&[1, 2, 99]),
        TaskAnyPoll::Unknown(99)
    ));
    assert!(
        matches!(scheduler.poll_any(&[1, 2, 3]), TaskAnyPoll::Failed(actual) if actual == error)
    );
}

#[test]
fn completion_racing_with_sleep_is_not_lost() {
    for _ in 0..100 {
        let scheduler = Arc::new(TaskScheduler::new());
        scheduler.register_result(1);
        let waiting = Arc::clone(&scheduler);
        let (send, receive) = mpsc::channel();
        let worker = std::thread::spawn(move || {
            waiting.wait_for_any(&[1]);
            send.send(()).expect("wake notification");
        });
        scheduler.store_result(1, Value::Unit);
        let woke = receive.recv_timeout(Duration::from_secs(2));
        scheduler.request_cancel();
        worker.join().expect("join");
        woke.expect("completion wakes waiter");
    }
}

#[test]
fn shutdown_releases_pending_wait() {
    let scheduler = Arc::new(TaskScheduler::new());
    scheduler.register_result(1);
    let waiting = Arc::clone(&scheduler);
    let worker = std::thread::spawn(move || waiting.wait_for_any(&[1]));
    scheduler.request_cancel();
    worker.join().expect("join");
    assert!(matches!(scheduler.poll_any(&[1]), TaskAnyPoll::Failed(_)));
}
