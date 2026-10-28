//! Task-bound function assignment keeps capture-cell destinations rejected.

use super::*;

fn call(name: &str, value: i64) -> DebugExpression {
    DebugExpression::Call {
        callee: Box::new(super::name(name)),
        arguments: vec![DebugExpression::Integer(value)],
    }
}

#[test]
fn task_bound_copy_rejects_capture_cell_destination_atomically() {
    let mut session = DebugSession::new(assignment_executable()).expect("debug session");
    stop_with_functions(&mut session);
    let captures = scope_reference(&mut session, "Captures");
    let generation = captures;
    let before_cell = named(
        &session
            .variables(captures, 0, 10)
            .expect("before cell")
            .items,
        "CellSlot",
    )
    .value
    .clone();

    let rejected = session
        .set_variable(captures, "CellSlot", &name("Bound"))
        .expect_err("task-bound capture-cell destination");
    assert_eq!(rejected.kind, DebugErrorKind::VariableValueType);
    assert!(
        rejected.message.contains("capture-cell") || rejected.hint.contains("captured mutable"),
        "{rejected:?}"
    );

    let preserved = session
        .variables(generation, 0, 10)
        .expect("generation survives capture-cell rejection")
        .items;
    assert_eq!(named(&preserved, "CellSlot").value, before_cell);

    let frame = session.stack(0, 1).expect("unchanged frame").items[0].id;
    session
        .set_expression(&root("Current"), &name("CellSlot"), Some(frame))
        .expect("non-task-bound cell source still copies");
    let frame = session.stack(0, 1).expect("after cell source").items[0].id;
    assert_eq!(
        session
            .evaluate(&call("Current", 1), Some(frame))
            .expect("cell payload")
            .value,
        "2"
    );
    session
        .set_expression(&root("Current"), &name("Bound"), Some(frame))
        .expect("same-frame register still accepts the task-bound source");
}
