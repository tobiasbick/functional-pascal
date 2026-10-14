use fpas_bytecode::SourceLocation;
use fpas_diagnostics::codes::RUNTIME_PROGRAM_PANIC;

use super::ScopeRegistry;
use crate::vm::runtime_error;

#[test]
fn scope_closes_only_after_every_child_completes_in_any_order() {
    let registry = ScopeRegistry::default();
    let (scope, _) = registry.open();
    for task in [10, 11, 12] {
        assert!(registry.enroll(scope, task).is_some());
    }

    assert!(registry.complete(12, None));
    assert!(registry.complete(10, None));
    assert!(registry.complete(10, None));
    assert!(registry.try_close(scope).is_none());

    assert!(registry.complete(11, None));
    let closed = registry.try_close(scope).expect("all children finished");
    assert_eq!(closed.tasks, vec![10, 11, 12]);
    assert!(closed.failures.is_empty());
    assert!(!registry.complete(11, None));
}

#[test]
fn failure_keeps_start_order_and_observation_sequence() {
    let registry = ScopeRegistry::default();
    let (scope, control) = registry.open();
    registry.enroll(scope, 20);
    registry.enroll(scope, 21);
    let error = runtime_error(
        RUNTIME_PROGRAM_PANIC,
        "boom",
        "Remove the panic.",
        SourceLocation::new(1, 1),
    );

    registry.complete(21, Some(&error));
    assert!(control.has_failed());
    assert!(registry.failure_sequence(21).is_some());
    assert!(registry.failure_sequence(20).is_none());

    registry.complete(20, None);
    let closed = registry.try_close(scope).expect("all children finished");
    assert_eq!(closed.failures.len(), 1);
    assert_eq!(closed.failures[0].start_order, 1);
}
