//! Pool worker smoke tests, concurrent enqueue stress, console mutex under threads.
//!
//! **Documentation:** `docs/future/parallel-vm.md`

use crate::vm::Worker;
use fpas_bytecode::Value;
use std::sync::Arc;
use std::thread;
use std::time::Duration;

use crate::tests::helpers::{loc, minimal_shared_state};

use super::shared_fixtures::{dummy_task, minimal_halt_chunk};

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
