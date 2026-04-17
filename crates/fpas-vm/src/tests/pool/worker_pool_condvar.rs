//! Shutdown wakes threads blocked on the task queue condvar.
//!
//! **Documentation:** `docs/future/parallel-vm.md` (Phase 4 checklist), `docs/pascal/08-concurrency.md`

use fpas_bytecode::{Chunk, Op};
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::thread;
use std::time::Duration;

use crate::tests::helpers::{loc, minimal_shared_state};

// --- Positive: shutdown wakes every condvar waiter (empty queue) ---

#[test]
fn request_shutdown_unblocks_multiple_threads_waiting_on_task_available() {
    let mut c = Chunk::new();
    c.emit(Op::Halt, loc());
    let shared = Arc::new(minimal_shared_state(c));

    let n = 4;
    let entered = Arc::new(AtomicUsize::new(0));
    let mut handles = Vec::new();

    for _ in 0..n {
        let s = Arc::clone(&shared);
        let cnt = Arc::clone(&entered);
        handles.push(thread::spawn(move || {
            let mut guard = s.task_queue.lock().unwrap_or_else(|e| e.into_inner());
            cnt.fetch_add(1, Ordering::SeqCst);
            while !s.is_shutdown() {
                guard = s
                    .task_available
                    .wait(guard)
                    .unwrap_or_else(|e| e.into_inner());
            }
        }));
    }

    let start = std::time::Instant::now();
    while entered.load(Ordering::SeqCst) < n {
        assert!(
            start.elapsed() <= Duration::from_secs(2),
            "waiters did not all block on condvar in time"
        );
        thread::yield_now();
    }
    thread::sleep(Duration::from_millis(20));

    shared.request_shutdown();

    for h in handles {
        h.join()
            .expect("each waiter thread must exit after notify_all + shutdown flag");
    }
    assert!(shared.is_shutdown());
}

// --- Negative / edge: shutdown before waiters start still terminates pool_loop-style wait ---

#[test]
fn pool_worker_style_wait_exits_immediately_if_shutdown_already_set() {
    let mut c = Chunk::new();
    c.emit(Op::Halt, loc());
    let shared = Arc::new(minimal_shared_state(c));
    shared.request_shutdown();

    let s2 = Arc::clone(&shared);
    let handle = thread::spawn(move || {
        let mut guard = s2.task_queue.lock().unwrap_or_else(|e| e.into_inner());
        while !s2.is_shutdown() {
            guard = s2
                .task_available
                .wait(guard)
                .unwrap_or_else(|e| e.into_inner());
        }
    });

    handle.join().expect("waiter should not block");
}
