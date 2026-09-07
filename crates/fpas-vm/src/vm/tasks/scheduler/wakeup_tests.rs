//! Shared wait signals for task results, cancellation, runnable work, and shutdown.

use super::{TaskScheduler, TaskState};
use crate::vm::cancellation::CancellationRegistry;
use crate::vm::shared::wakeups::WakeSignal;
use crate::vm::{TaskAnyPoll, TaskResultPoll};
use fpas_bytecode::{FunctionId, Value};
use std::sync::Arc;
use std::time::Duration;

#[test]
fn either_task_completion_or_cancellation_wakes_the_same_signal() {
    let scheduler = TaskScheduler::new();
    let cancellations = CancellationRegistry::new();
    for cancel in [false, true] {
        let id = scheduler.alloc_id();
        scheduler.register_result(id);
        let token = cancellations.create_source();
        let signal = WakeSignal::new();
        let task_registration = scheduler.subscribe(&signal);
        let cancellation_registration = cancellations.subscribe(token, &signal).expect("subscribe");
        assert!(matches!(scheduler.poll_any(&[id]), TaskAnyPoll::Pending));
        assert!(!cancellations.is_cancelled(token).expect("probe"));
        if cancel {
            cancellations.cancel(token).expect("cancel");
        } else {
            scheduler.store_result(id, Value::Integer(42));
        }
        assert!(signal.wait(Duration::ZERO));
        if cancel {
            assert!(matches!(scheduler.poll_result(id), TaskResultPoll::Pending));
        } else {
            assert!(matches!(
                scheduler.poll_result(id),
                TaskResultPoll::Available(Value::Integer(42))
            ));
        }
        drop((task_registration, cancellation_registration));
        assert_eq!(scheduler.changes.registration_count(), 0);
    }
}

#[test]
fn queued_work_wakes_a_registered_helper_without_consuming_the_task() {
    let scheduler = TaskScheduler::new();
    let signal = WakeSignal::new();
    let _registration = scheduler.subscribe(&signal);
    assert!(scheduler.try_dequeue().is_none());
    scheduler.enqueue(TaskState {
        suspension: None,
        supervision: None,
        id: 9,
        function: FunctionId::new(0),
        ip: 0,
        base: 0,
        registers: vec![],
        register_initialized: vec![],
        frames: vec![],
        retain_result: false,
        instruction_count: 0,
        suppressed_initializers: vec![],
        callback_continuations: vec![],
    });
    assert!(signal.wait(Duration::ZERO));
    assert_eq!(scheduler.try_dequeue().expect("retained work").id, 9);
}

#[test]
fn failure_and_shutdown_notify_without_consuming_the_original_error() {
    for shutdown in [false, true] {
        let scheduler = TaskScheduler::new();
        scheduler.register_result(1);
        let signal = WakeSignal::new();
        let _registration = scheduler.subscribe(&signal);
        let error = super::tests::failure("original failure");
        if shutdown {
            scheduler.fail(error.clone());
        } else {
            scheduler.store_failure(1, error.clone());
        }
        assert!(signal.wait(Duration::ZERO));
        assert!(
            matches!(scheduler.poll_result(1), TaskResultPoll::Failed(actual) if actual == error)
        );
    }
}

#[test]
fn completion_racing_with_registration_is_observed_and_registrations_are_removed() {
    let scheduler = Arc::new(TaskScheduler::new());
    for id in 1..=100 {
        scheduler.register_result(id);
        let completing = Arc::clone(&scheduler);
        let notifier = std::thread::spawn(move || completing.store_result(id, Value::Unit));
        let signal = WakeSignal::new();
        let registration = scheduler.subscribe(&signal);
        let observed = matches!(scheduler.poll_any(&[id]), TaskAnyPoll::Complete(0))
            || signal.wait(Duration::from_secs(2));
        notifier.join().expect("join notifier");
        drop(registration);
        assert!(observed, "neither probe nor wake observed the completion");
        assert_eq!(scheduler.changes.registration_count(), 0);
    }
}
