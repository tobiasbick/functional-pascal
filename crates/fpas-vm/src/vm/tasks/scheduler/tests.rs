//! Exact retained-failure transition tests.

use fpas_bytecode::{SourceLocation, Value};
use fpas_diagnostics::codes::{RUNTIME_PROGRAM_PANIC, RUNTIME_VM_SHUTDOWN};

use super::TaskScheduler;
use crate::vm::{TaskBatchPoll, TaskResultPoll, runtime_error};

pub(super) fn failure(message: &str) -> crate::vm::VmError {
    runtime_error(
        RUNTIME_PROGRAM_PANIC,
        message,
        "Recover explicitly in the debugger.",
        SourceLocation::new(1, 1),
    )
}

#[test]
fn retained_failure_transitions_require_the_exact_diagnostic() {
    let scheduler = TaskScheduler::new();
    scheduler.register_result(1);
    let original = failure("original");
    scheduler.store_failure(1, original.clone());

    assert!(!scheduler.recover_failure(1, &failure("different")));
    assert!(matches!(scheduler.poll_result(1), TaskResultPoll::Failed(error) if error == original));
    assert!(scheduler.recover_failure(1, &original));
    assert!(matches!(scheduler.poll_result(1), TaskResultPoll::Pending));

    scheduler.store_failure(1, original.clone());
    assert!(!scheduler.replace_failure(1, &failure("different"), Value::Integer(9)));
    assert!(matches!(scheduler.poll_result(1), TaskResultPoll::Failed(error) if error == original));
    assert!(scheduler.replace_failure(1, &original, Value::Integer(9)));
    assert!(matches!(
        scheduler.poll_result(1),
        TaskResultPoll::Available(Value::Integer(9))
    ));
    assert!(matches!(scheduler.poll_result(1), TaskResultPoll::Consumed));
}

#[test]
fn shutdown_completes_a_pending_retained_result() {
    let scheduler = TaskScheduler::new();
    scheduler.register_result(7);
    scheduler.request_cancel();

    scheduler.fail_pending_result_if_shutdown(7);

    assert!(matches!(
        scheduler.poll_result(7),
        TaskResultPoll::Failed(error) if error.code == RUNTIME_VM_SHUTDOWN
    ));
}

#[test]
fn run_failure_is_preserved_when_shutdown_completes_other_pending_results() {
    let scheduler = TaskScheduler::new();
    scheduler.register_result(1);
    scheduler.register_result(2);
    let original = failure("worker panic");
    scheduler.store_failure(2, original.clone());
    scheduler.fail(original.clone());

    assert!(matches!(
        scheduler.poll_result(1),
        TaskResultPoll::Failed(error) if error == original
    ));
    assert!(matches!(
        scheduler.poll_result(2),
        TaskResultPoll::Failed(error) if error == original
    ));
}

#[test]
fn late_success_does_not_replace_a_shutdown_failure() {
    let scheduler = TaskScheduler::new();
    scheduler.register_result(3);
    scheduler.request_cancel();

    scheduler.store_result(3, Value::Integer(99));

    assert!(matches!(
        scheduler.poll_result(3),
        TaskResultPoll::Failed(error) if error.code == RUNTIME_VM_SHUTDOWN
    ));
}

#[test]
fn sequential_consumed_task_ids_use_one_completion_range() {
    let scheduler = TaskScheduler::new();

    for _ in 0..10_000 {
        let task_id = scheduler.alloc_id();
        scheduler.register_result(task_id);
        scheduler.store_result(task_id, Value::Unit);
        assert!(matches!(
            scheduler.poll_result(task_id),
            TaskResultPoll::Available(Value::Unit)
        ));
    }

    assert_eq!(scheduler.consumed_completion_storage_len(), 1);
}

#[test]
fn consumed_task_id_remains_complete_for_wait_all() {
    let scheduler = TaskScheduler::new();
    scheduler.register_result(1);
    scheduler.store_result(1, Value::Unit);
    assert!(matches!(
        scheduler.poll_result(1),
        TaskResultPoll::Available(Value::Unit)
    ));

    assert!(matches!(
        scheduler.poll_batch(&[1]),
        TaskBatchPoll::Complete
    ));
}
