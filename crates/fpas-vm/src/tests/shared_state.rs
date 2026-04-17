//! Cross-thread [`SharedState`](crate::vm::SharedState): ready queue, task results, shutdown, I/O mutexes.
//!
//! **Documentation:** `docs/future/parallel-vm.md`

use crate::vm::{TaskResultPoll, TaskState, Worker};
use fpas_bytecode::{Chunk, Op, Value};
use std::sync::Arc;
use std::sync::atomic::Ordering;
use std::thread;
use std::time::Duration;

use super::helpers::{loc, minimal_shared_state};

fn minimal_halt_chunk() -> Chunk {
    let mut chunk = Chunk::new();
    chunk.emit(Op::Halt, loc());
    chunk
}

fn dummy_task(id: u64, ip: usize) -> TaskState {
    TaskState {
        id,
        ip,
        stack: Vec::new(),
        call_stack: Vec::new(),
        retain_result: false,
    }
}

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

// --- Positive: shutdown ---

#[test]
fn request_shutdown_sets_flag_and_is_idempotent() {
    let shared = minimal_shared_state(minimal_halt_chunk());
    assert!(!shared.is_shutdown());
    shared.request_shutdown();
    assert!(shared.is_shutdown());
    shared.request_shutdown();
    assert!(shared.is_shutdown());
    assert!(shared.shutdown.load(Ordering::Acquire));
}

// --- Positive: condvar progress wait ---

#[test]
fn wait_for_task_progress_returns_on_timeout_without_progress() {
    let shared = Arc::new(minimal_shared_state(minimal_halt_chunk()));
    let start = std::time::Instant::now();
    shared.wait_for_task_progress(Some(Duration::from_millis(40)));
    assert!(
        start.elapsed() < Duration::from_millis(500),
        "should not block indefinitely"
    );
}

#[test]
fn wait_for_task_progress_wakes_on_enqueue_notify() {
    let shared = Arc::new(minimal_shared_state(minimal_halt_chunk()));
    let s2 = Arc::clone(&shared);
    let waiter = thread::spawn(move || {
        s2.wait_for_task_progress(Some(Duration::from_secs(5)));
    });
    thread::sleep(Duration::from_millis(30));
    shared.enqueue_task(dummy_task(1, 0));
    waiter.join().expect("waiter join");
}

#[test]
fn wait_for_task_progress_wakes_on_store_task_result() {
    let shared = Arc::new(minimal_shared_state(minimal_halt_chunk()));
    let s2 = Arc::clone(&shared);
    let waiter = thread::spawn(move || {
        s2.wait_for_task_progress(None);
    });
    thread::sleep(Duration::from_millis(30));
    shared.store_task_result(42, Value::Integer(1));
    waiter.join().expect("waiter join");
}

// --- Positive: pool worker integration (queue + condvar + shutdown) ---

#[test]
fn pool_worker_drains_prequeued_task_then_exits_on_shutdown() {
    let chunk = minimal_halt_chunk();
    let shared = Arc::new(minimal_shared_state(chunk));
    shared.enqueue_task(dummy_task(10, 0));

    let s2 = Arc::clone(&shared);
    let handle = thread::spawn(move || {
        let mut w = Worker::new_pool(s2);
        w.pool_loop().expect("pool loop ok");
    });

    thread::sleep(Duration::from_millis(100));
    shared.request_shutdown();
    handle.join().expect("pool thread join");
    assert!(shared.is_shutdown());
}

#[test]
fn pool_worker_blocks_until_enqueue_then_shutdown() {
    let chunk = minimal_halt_chunk();
    let shared = Arc::new(minimal_shared_state(chunk));

    let s2 = Arc::clone(&shared);
    let handle = thread::spawn(move || {
        let mut w = Worker::new_pool(s2);
        w.pool_loop().expect("pool loop ok");
    });

    thread::sleep(Duration::from_millis(40));
    shared.enqueue_task(dummy_task(11, 0));
    thread::sleep(Duration::from_millis(40));
    shared.request_shutdown();
    handle.join().expect("pool thread join");
}

// --- Stress / edge: concurrent enqueue ---

#[test]
fn concurrent_enqueues_all_dequeued() {
    let shared = Arc::new(minimal_shared_state(minimal_halt_chunk()));
    let mut handles = Vec::new();
    for t in 0..8 {
        let s = Arc::clone(&shared);
        handles.push(thread::spawn(move || {
            for i in 0..25 {
                let id = t * 1000 + i;
                s.enqueue_task(dummy_task(id, 0));
            }
        }));
    }
    for h in handles {
        h.join().expect("producer join");
    }

    let mut seen = 0u32;
    while shared.try_dequeue_task().is_some() {
        seen += 1;
    }
    assert_eq!(seen, 8 * 25);
}

// --- Positive: I/O mutexes do not panic under concurrent access ---

#[test]
fn console_lock_serializes_concurrent_writes() {
    let shared = Arc::new(minimal_shared_state(minimal_halt_chunk()));
    let mut handles = Vec::new();
    for k in 0..6 {
        let s = Arc::clone(&shared);
        handles.push(thread::spawn(move || {
            for _ in 0..20 {
                let mut c = s.console.lock().unwrap_or_else(|e| e.into_inner());
                c.write_ln(&Value::Str(format!("line-{k}")), loc())
                    .expect("write_ln");
            }
        }));
    }
    for h in handles {
        h.join().expect("writer join");
    }
    let lines = shared
        .console
        .lock()
        .unwrap_or_else(|e| e.into_inner())
        .output()
        .lines
        .len();
    assert_eq!(lines, 6 * 20);
}
