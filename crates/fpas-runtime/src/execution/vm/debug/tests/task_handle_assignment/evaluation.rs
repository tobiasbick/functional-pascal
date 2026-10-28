//! Evaluation-boundary regressions for debugger task-handle assignment.

use super::super::*;
use super::fixtures::*;
use super::support::*;

#[test]
fn textual_assignment_works_after_a_global_task_handle_is_initialized() {
    let mut session = DebugSession::new(assignment_executable()).expect("debug session");
    stop_with_tasks(&mut session);
    let frame = session.stack(0, 1).expect("stack").items[0].id;
    session
        .set_expression(&root("G"), &name("Pending"), Some(frame))
        .expect("initialize global task handle");

    let frame = session.stack(0, 1).expect("fresh stack").items[0].id;
    session
        .set_expression(&root("Current"), &name("Pending"), Some(frame))
        .expect("copy while a global task handle exists");
}

#[test]
fn indexed_task_assignment_shares_selector_and_source_operation_budget() {
    let mut session = DebugSession::new(assignment_executable()).expect("debug session");
    stop_with_tasks(&mut session);
    let target = DebugAssignmentTarget {
        root: "Items".to_string(),
        selectors: vec![DebugAssignmentSelector::Index(DebugExpression::Integer(0))],
    };
    let frame = session.stack(0, 1).expect("stack").items[0].id;

    let exhausted = session
        .set_expression_with_limits(
            &target,
            &name("Pending"),
            Some(frame),
            DebugEvaluationLimits {
                max_operations: 1,
                ..DebugEvaluationLimits::default()
            },
        )
        .expect_err("selector and source require two operations");
    assert_eq!(exhausted.kind, DebugErrorKind::EvaluationLimit);

    session
        .set_expression_with_limits(
            &target,
            &name("Pending"),
            Some(frame),
            DebugEvaluationLimits {
                max_operations: 2,
                ..DebugEvaluationLimits::default()
            },
        )
        .expect("two operations are sufficient");
}
