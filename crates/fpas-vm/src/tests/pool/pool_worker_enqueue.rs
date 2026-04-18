//! Pool worker wakes on `enqueue_task` and drains multiple pre-queued tasks.
//!
//! **Documentation:** `docs/rust/parallel-vm.md` (Phase 5 checklist), `docs/pascal/08-concurrency.md`

use crate::vm::{TaskState, Worker};
use fpas_bytecode::Value;
use std::sync::Arc;
use std::thread;
use std::time::Duration;

use crate::tests::helpers::minimal_shared_state;

use super::pool_worker_common::{chunk_task_returns_integer, wait_for_task_result};

// --- Positive: enqueue wakes a pool worker blocked on an empty queue ---

#[test]
fn enqueue_task_unblocks_pool_loop_waiting_on_condvar() {
    let chunk = chunk_task_returns_integer(41);
    let shared = Arc::new(minimal_shared_state(chunk));

    let s_loop = Arc::clone(&shared);
    let pool = thread::spawn(move || {
        let mut w = Worker::new_pool(s_loop);
        w.pool_loop()
    });

    thread::sleep(Duration::from_millis(40));

    shared.enqueue_task(TaskState {
        id: 1,
        ip: 0,
        stack: Vec::new(),
        call_stack: Vec::new(),
        retain_result: true,
    });

    let v = wait_for_task_result(&shared, 1, Duration::from_secs(2));
    assert_eq!(v, Value::Integer(41));

    shared.request_shutdown();
    pool.join()
        .expect("pool thread joins")
        .expect("pool_loop ok");
}

// --- Positive: fast dequeue path drains multiple tasks without extra waits ---

#[test]
fn pool_loop_drains_multiple_prequeued_tasks_before_blocking_again() {
    let chunk = chunk_task_returns_integer(2);
    let shared = Arc::new(minimal_shared_state(chunk));

    shared.enqueue_task(TaskState {
        id: 1,
        ip: 0,
        stack: Vec::new(),
        call_stack: Vec::new(),
        retain_result: true,
    });
    shared.enqueue_task(TaskState {
        id: 2,
        ip: 0,
        stack: Vec::new(),
        call_stack: Vec::new(),
        retain_result: true,
    });

    let s_loop = Arc::clone(&shared);
    let pool = thread::spawn(move || {
        let mut w = Worker::new_pool(s_loop);
        w.pool_loop()
    });

    assert_eq!(
        wait_for_task_result(&shared, 1, Duration::from_secs(2)),
        Value::Integer(2)
    );
    assert_eq!(
        wait_for_task_result(&shared, 2, Duration::from_secs(2)),
        Value::Integer(2)
    );

    shared.request_shutdown();
    pool.join().expect("join").expect("pool_loop ok");
}
