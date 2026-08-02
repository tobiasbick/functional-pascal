//! Shutdown flag and [`SharedState::wait_for_task_progress`](crate::vm::SharedState::wait_for_task_progress).
//!
//! **Documentation:** `docs/pascal/language/concurrency/README.md`

use crate::vm::TaskTimers;
use fpas_bytecode::Value;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc;
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
fn wait_until_task_result_ready_wakes_on_store_task_result() {
    let shared = Arc::new(minimal_shared_state(minimal_halt_chunk()));
    let s2 = Arc::clone(&shared);
    let waiter = thread::spawn(move || {
        s2.wait_until_task_result_ready(42);
    });
    thread::sleep(Duration::from_millis(30));
    shared.store_task_result(42, Value::Integer(1));
    waiter.join().expect("waiter join");
}

#[test]
fn timer_handoff_finishes_before_cancellation_can_drain() {
    let timers = Arc::new(TaskTimers::new());
    let accepting_tasks = AtomicBool::new(true);
    assert!(
        timers
            .schedule(dummy_task(17, 0), 0, &accepting_tasks)
            .is_ok()
    );

    let shutdown = Arc::new(AtomicBool::new(false));
    let (handoff_started_tx, handoff_started_rx) = mpsc::sync_channel(1);
    let (release_handoff_tx, release_handoff_rx) = mpsc::sync_channel(1);
    let dispatch_timers = Arc::clone(&timers);
    let dispatch_shutdown = Arc::clone(&shutdown);
    let dispatcher = thread::spawn(move || {
        assert!(
            dispatch_timers.dispatch_next_due(&dispatch_shutdown, |tasks| {
                assert_eq!(tasks.len(), 1);
                assert_eq!(tasks[0].id, 17);
                handoff_started_tx
                    .send(())
                    .expect("handoff observer should remain connected");
                release_handoff_rx
                    .recv()
                    .expect("test should release the handoff");
            })
        );
    });
    handoff_started_rx
        .recv_timeout(Duration::from_secs(1))
        .expect("due timer should enter its dispatch handoff");

    let (cancelled_tx, cancelled_rx) = mpsc::sync_channel(1);
    let cancel_timers = Arc::clone(&timers);
    let canceller = thread::spawn(move || {
        cancelled_tx
            .send(cancel_timers.cancel_all())
            .expect("cancellation observer should remain connected");
    });
    assert!(
        cancelled_rx
            .recv_timeout(Duration::from_millis(40))
            .is_err(),
        "cancellation must not overtake a removed bucket's ready-queue handoff"
    );

    release_handoff_tx
        .send(())
        .expect("dispatcher should remain connected");
    dispatcher.join().expect("dispatcher should not panic");
    let cancelled = cancelled_rx
        .recv_timeout(Duration::from_secs(1))
        .expect("cancellation should resume after the handoff");
    canceller.join().expect("canceller should not panic");
    assert!(cancelled.is_empty());
}
