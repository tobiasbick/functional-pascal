//! [`SharedState`](crate::vm::SharedState): task ids, ready queue, task result polling.
//!
//! **Documentation:** `docs/pascal/language/concurrency/README.md`

use crate::vm::{TaskBatchPoll, TaskResultPoll, TaskTimers};
use fpas_bytecode::Value;
use std::sync::atomic::AtomicBool;

use crate::tests::helpers::minimal_shared_state;

use super::shared_fixtures::{dummy_task, minimal_halt_chunk};

// --- Positive: task id allocation ---

#[test]
fn alloc_task_id_starts_at_one_and_increments() {
    let shared = minimal_shared_state(minimal_halt_chunk());
    assert_eq!(shared.alloc_task_id(), 1);
    assert_eq!(shared.alloc_task_id(), 2);
    assert_eq!(shared.alloc_task_id(), 3);
}

// --- Positive / edge: queue ---

#[test]
fn try_dequeue_empty_returns_none() {
    let shared = minimal_shared_state(minimal_halt_chunk());
    assert!(shared.try_dequeue_task().is_none());
}

#[test]
fn enqueue_then_try_dequeue_returns_same_task() {
    let shared = minimal_shared_state(minimal_halt_chunk());
    let task = dummy_task(7, 3);
    shared.enqueue_task(task);
    let got = shared.try_dequeue_task().expect("one task");
    assert_eq!(got.id, 7);
    assert_eq!(got.ip, 3);
    assert!(shared.try_dequeue_task().is_none());
}

#[test]
fn ready_queue_is_fifo_under_single_threaded_push_pop() {
    let shared = minimal_shared_state(minimal_halt_chunk());
    shared.enqueue_task(dummy_task(1, 0));
    shared.enqueue_task(dummy_task(2, 0));
    assert_eq!(shared.try_dequeue_task().unwrap().id, 1);
    assert_eq!(shared.try_dequeue_task().unwrap().id, 2);
    assert!(shared.try_dequeue_task().is_none());
}

// --- Positive / negative / edge: task results ---

#[test]
fn poll_task_result_reports_unknown_id() {
    let shared = minimal_shared_state(minimal_halt_chunk());
    assert!(matches!(
        shared.poll_task_result(999),
        TaskResultPoll::Unknown
    ));
}

#[test]
fn registered_task_result_is_pending_until_stored() {
    let shared = minimal_shared_state(minimal_halt_chunk());
    shared.register_task_result(5);
    assert!(matches!(
        shared.poll_task_result(5),
        TaskResultPoll::Pending
    ));
    assert!(matches!(
        shared.poll_task_batch(&[5]),
        TaskBatchPoll::Pending
    ));
}

#[test]
fn store_poll_available_then_consumed() {
    let shared = minimal_shared_state(minimal_halt_chunk());
    let v = Value::Integer(42);
    shared.store_task_result(5, v.clone());

    assert!(matches!(
        shared.poll_task_batch(&[5]),
        TaskBatchPoll::Complete
    ));

    assert!(matches!(
        shared.poll_task_result(5),
        TaskResultPoll::Available(ref got) if *got == v
    ));

    assert!(matches!(
        shared.poll_task_result(5),
        TaskResultPoll::Consumed
    ));

    assert!(matches!(
        shared.poll_task_batch(&[5]),
        TaskBatchPoll::Complete
    ));
}

#[test]
fn poll_never_available_without_store() {
    let shared = minimal_shared_state(minimal_halt_chunk());
    shared.register_task_result(1);
    for _ in 0..3 {
        assert!(matches!(
            shared.poll_task_result(1),
            TaskResultPoll::Pending
        ));
    }
}

#[test]
fn normal_main_teardown_explicitly_fails_retained_sleepers() {
    let shared = minimal_shared_state(minimal_halt_chunk());
    let task_id = 7;
    shared.register_task_result(task_id);
    let mut task = dummy_task(task_id, 0);
    task.retain_result = true;
    shared.schedule_task_after(task, 60_000);

    shared.finish_main_task();

    let TaskResultPoll::Failed(error) = shared.poll_task_result(task_id) else {
        panic!("retained sleeper should have an explicit cancellation result");
    };
    assert!(error.message.contains("main task finished"));
    assert!(shared.is_shutdown());
}

#[test]
fn timer_cancellation_removes_detached_sleepers_explicitly() {
    let timers = TaskTimers::new();
    let accepting_tasks = AtomicBool::new(true);
    let task = dummy_task(9, 0);
    assert!(!task.retain_result);
    assert!(timers.schedule(task, 60_000, &accepting_tasks).is_ok());

    let cancelled = timers.cancel_all();

    assert_eq!(cancelled.len(), 1);
    assert_eq!(cancelled[0].id, 9);
    assert!(!cancelled[0].retain_result);
    assert!(timers.cancel_all().is_empty());
}

#[test]
fn task_batch_snapshot_cannot_treat_failure_as_successful_completion() {
    let shared = minimal_shared_state(minimal_halt_chunk());
    shared.register_task_result(1);
    shared.register_task_result(2);
    shared.store_task_result(1, Value::Unit);
    shared.store_task_failure(
        2,
        crate::vm::internal_error(
            "spawned task failed",
            "test failure",
            crate::tests::helpers::loc(),
        ),
    );

    let TaskBatchPoll::Failed(error) = shared.poll_task_batch(&[1, 2]) else {
        panic!("a failed WaitAll member must outrank successful completion");
    };
    assert_eq!(error.message, "spawned task failed");
}
