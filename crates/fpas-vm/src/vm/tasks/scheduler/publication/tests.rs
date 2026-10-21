//! Low-level publication remains safe even when a caller bypasses worker checks.

use fpas_bytecode::{MemoryDomain, Value};
use fpas_diagnostics::codes::RUNTIME_INVALID_TASK;

use crate::vm::TaskResultPoll;
use crate::vm::tasks::scheduler::{RetainedResultReplacement, TaskScheduler, tests::failure};

#[test]
fn forbidden_results_complete_as_owned_failures_without_pending_results() {
    for wrapped in [false, true] {
        let scheduler = TaskScheduler::new();
        let (scope, _) = scheduler.scopes.open();
        scheduler.scopes.enroll(scope, 1).expect("membership");
        scheduler.register_result(1);
        let domain = MemoryDomain::new();
        let local = Value::Cell(domain.new_cell(Value::Integer(1)).expect("cell"));
        let value = if wrapped {
            Value::result_ok(Value::Array(vec![local].into()))
        } else {
            local
        };
        scheduler.store_result(1, value);
        assert!(
            matches!(scheduler.poll_result(1), TaskResultPoll::Failed(error) if error.code == RUNTIME_INVALID_TASK)
        );
        assert!(!scheduler.is_shutdown(), "owned failure stays contained");
        let report = scheduler.poll_scope_close(scope).expect("scope completed");
        assert_eq!(report.len(), 1);
    }
}

#[test]
fn invalid_debug_replacements_preserve_the_previous_result_or_failure() {
    let scheduler = TaskScheduler::new();
    let domain = MemoryDomain::new();
    let local = Value::option_some(Value::Cell(domain.new_cell(Value::Unit).expect("cell")));
    scheduler.register_result(1);
    scheduler.store_result(1, Value::Integer(42));
    assert!(matches!(
        scheduler.replace_available_result(1, local.clone()),
        RetainedResultReplacement::Invalid(_)
    ));
    assert!(matches!(
        scheduler.poll_result(1),
        TaskResultPoll::Available(Value::Integer(42))
    ));

    scheduler.register_result(2);
    let original = failure("original");
    scheduler.store_failure(2, original.clone());
    assert!(!scheduler.replace_failure(2, &original, local));
    assert!(matches!(scheduler.poll_result(2), TaskResultPoll::Failed(error) if error == original));
}

#[test]
fn immutable_nested_results_remain_available() {
    let scheduler = TaskScheduler::new();
    let value = Value::result_ok(Value::Array(vec![Value::Integer(42)].into()));
    scheduler.register_result(1);
    scheduler.store_result(1, value.clone());
    assert!(
        matches!(scheduler.poll_result(1), TaskResultPoll::Available(actual) if actual == value)
    );
}

#[test]
fn late_invalid_results_preserve_terminal_outcomes_without_aborting() {
    let scheduler = TaskScheduler::new();
    scheduler.register_result(1);
    scheduler.store_result(1, Value::Integer(42));
    let domain = MemoryDomain::new();
    scheduler.store_result(1, Value::Cell(domain.new_cell(Value::Unit).expect("cell")));
    assert!(!scheduler.is_shutdown());
    assert!(matches!(
        scheduler.poll_result(1),
        TaskResultPoll::Available(Value::Integer(42))
    ));
}
