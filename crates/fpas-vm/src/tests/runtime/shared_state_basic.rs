//! [`SharedState`](crate::vm::SharedState): task ids, ready queue, task result polling.
//!
//! **Documentation:** `docs/rust/parallel-vm.md`

use crate::vm::TaskResultPoll;
use fpas_bytecode::Value;

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
fn ready_queue_is_lifo_under_single_threaded_push_pop() {
    let shared = minimal_shared_state(minimal_halt_chunk());
    shared.enqueue_task(dummy_task(1, 0));
    shared.enqueue_task(dummy_task(2, 0));
    assert_eq!(shared.try_dequeue_task().unwrap().id, 2);
    assert_eq!(shared.try_dequeue_task().unwrap().id, 1);
    assert!(shared.try_dequeue_task().is_none());
}

// --- Positive / negative / edge: task results ---

#[test]
fn poll_task_result_pending_for_unknown_id() {
    let shared = minimal_shared_state(minimal_halt_chunk());
    assert!(matches!(
        shared.poll_task_result(999),
        TaskResultPoll::Pending
    ));
}

#[test]
fn store_poll_available_then_consumed() {
    let shared = minimal_shared_state(minimal_halt_chunk());
    let v = Value::Integer(42);
    shared.store_task_result(5, v.clone());

    assert!(shared.task_completion_recorded(5));

    assert!(matches!(
        shared.poll_task_result(5),
        TaskResultPoll::Available(ref got) if *got == v
    ));

    assert!(matches!(
        shared.poll_task_result(5),
        TaskResultPoll::Consumed
    ));

    assert!(
        shared.task_completion_recorded(5),
        "completion remains recorded after consume so Wait semantics can detect finished tasks"
    );
}

#[test]
fn poll_never_available_without_store() {
    let shared = minimal_shared_state(minimal_halt_chunk());
    for _ in 0..3 {
        assert!(matches!(
            shared.poll_task_result(1),
            TaskResultPoll::Pending
        ));
    }
}
