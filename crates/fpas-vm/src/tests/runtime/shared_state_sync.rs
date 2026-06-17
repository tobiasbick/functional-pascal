//! Shutdown flag and [`SharedState::wait_for_task_progress`](crate::vm::SharedState::wait_for_task_progress).
//!
//! **Documentation:** `docs/pascal/08-concurrency.md`

use fpas_bytecode::Value;
use std::sync::Arc;
use std::sync::atomic::Ordering;
use std::thread;
use std::time::Duration;

use crate::tests::helpers::minimal_shared_state;

use super::shared_fixtures::{dummy_task, minimal_halt_chunk};

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
