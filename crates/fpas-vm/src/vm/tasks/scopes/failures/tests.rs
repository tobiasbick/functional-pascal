use fpas_bytecode::SourceLocation;
use fpas_diagnostics::codes::{RUNTIME_PROGRAM_PANIC, RUNTIME_TASK_CANCELLED};

use super::{ScopeFailure, scope_outcome};
use crate::vm::{VmError, runtime_error};

fn panic_error(message: &str) -> VmError {
    runtime_error(
        RUNTIME_PROGRAM_PANIC,
        message,
        "Remove the panic.",
        SourceLocation::new(3, 5),
    )
}

fn failure(sequence: u64, start_order: usize, message: &str) -> ScopeFailure {
    ScopeFailure {
        sequence,
        start_order,
        error: panic_error(message),
    }
}

fn cancelled() -> VmError {
    runtime_error(
        RUNTIME_TASK_CANCELLED,
        "cancelled",
        "Wait inside the scope.",
        SourceLocation::new(1, 1),
    )
}

#[test]
fn no_failures_and_no_body_error_end_normally() {
    assert!(scope_outcome(Vec::new(), None).is_none());
}

#[test]
fn first_observed_child_failure_is_primary_and_others_follow_in_start_order() {
    let (sequence, error) = scope_outcome(
        vec![
            failure(9, 0, "first started"),
            failure(4, 2, "first observed"),
            failure(7, 1, "second started"),
        ],
        None,
    )
    .expect("failure");
    assert_eq!(sequence, 4);
    assert_eq!(error.message, "first observed");
    let help = error.help.expect("help");
    let first = help.find("first started").expect("first started attached");
    let second = help
        .find("second started")
        .expect("second started attached");
    assert!(first < second, "{help}");
    assert!(
        help.contains("Further failures in this task scope (2)"),
        "{help}"
    );
}

#[test]
fn body_cancellation_yields_to_child_failure() {
    let (_, error) =
        scope_outcome(vec![failure(5, 0, "child")], Some((6, cancelled()))).expect("failure");
    assert_eq!(error.message, "child");
    assert!(!error.help.unwrap_or_default().contains("Further"));
}

#[test]
fn body_cancellation_without_child_failure_is_reported_as_is() {
    let (_, error) = scope_outcome(Vec::new(), Some((2, cancelled()))).expect("error");
    assert_eq!(error.code, RUNTIME_TASK_CANCELLED);
}

#[test]
fn body_failure_observed_first_is_primary() {
    let (sequence, error) =
        scope_outcome(vec![failure(8, 0, "child")], Some((3, panic_error("body"))))
            .expect("failure");
    assert_eq!(sequence, 3);
    assert_eq!(error.message, "body");
    assert!(error.help.expect("help").contains("child"));
}

#[test]
fn child_failure_observed_through_wait_is_reported_once() {
    let child = failure(2, 0, "child");
    let (_, error) = scope_outcome(vec![child.clone()], Some((2, child.error))).expect("failure");
    assert_eq!(error.message, "child");
    assert!(!error.help.unwrap_or_default().contains("Further"));
}

#[test]
fn independent_body_and_child_failures_with_equal_diagnostics_are_both_reported() {
    let child = failure(5, 0, "same panic");
    let (sequence, error) =
        scope_outcome(vec![child.clone()], Some((2, child.error))).expect("failure");
    assert_eq!(sequence, 2, "the earlier body failure stays primary");
    assert!(
        error
            .help
            .expect("help")
            .contains("Further failures in this task scope (1)")
    );
}
