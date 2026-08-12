//! Session-level uninitialized root assignment coverage.

use super::*;

#[test]
fn initialized_unit_is_not_rendered_as_the_empty_sentinel() {
    let mut session = DebugSession::new(assignment_executable()).expect("debug session");
    let locals = scope_reference(&mut session, "Locals");
    assert_eq!(
        named(
            &session.variables(locals, 0, 10).expect("entry").items,
            "UnitValue"
        )
        .value,
        "<uninitialized>"
    );
    stopped(session.step_into().expect("execute LoadUnit"));
    let locals = scope_reference(&mut session, "Locals");
    assert_eq!(
        named(
            &session.variables(locals, 0, 10).expect("after unit").items,
            "UnitValue"
        )
        .value,
        "()"
    );
}

#[test]
fn mutable_local_and_global_roots_initialize_from_handles_and_names() {
    let mut session = DebugSession::new(assignment_executable()).expect("debug session");
    let locals = scope_reference(&mut session, "Locals");
    assert_eq!(
        named(
            &session
                .variables(locals, 0, 10)
                .expect("entry locals")
                .items,
            "Count"
        )
        .value,
        "<uninitialized>"
    );

    let updated = session
        .set_variable(locals, "Count", &DebugExpression::Integer(30))
        .expect("handle initialization");
    assert_eq!(updated.value, "30");
    let frame = session.stack(0, 1).expect("fresh stack").items[0].id;
    let locals = scope_reference(&mut session, "Locals");
    assert_eq!(
        named(
            &session.variables(locals, 0, 10).expect("locals").items,
            "Count"
        )
        .value,
        "30"
    );

    session
        .set_expression(&root("cOuNt"), &DebugExpression::Integer(8), Some(frame))
        .expect("textual mixed-case initialization");
    let globals = scope_reference(&mut session, "Globals");
    assert_eq!(
        named(
            &session
                .variables(globals, 0, 10)
                .expect("entry globals")
                .items,
            "G"
        )
        .value,
        "<uninitialized>"
    );
    assert_eq!(
        session
            .set_variable(globals, "G", &DebugExpression::Integer(4))
            .expect("global handle initialization")
            .value,
        "4"
    );
    session
        .set_expression(&root("G"), &DebugExpression::Integer(7), None)
        .expect("textual global replacement");
    let frame = session.stack(0, 1).expect("after global").items[0].id;
    assert_eq!(
        session
            .evaluate(&DebugExpression::Name("Count".to_string()), Some(frame))
            .expect("initialized local")
            .value,
        "8"
    );
    assert_eq!(
        session
            .evaluate(&DebugExpression::Name("G".to_string()), None)
            .expect("replaced global")
            .value,
        "7"
    );
}

#[test]
fn source_initializer_overwrites_debugger_initialization_after_continue() {
    let mut session = DebugSession::new(assignment_executable()).expect("debug session");
    let frame = session.stack(0, 1).expect("stack").items[0].id;
    session
        .set_expression(&root("Count"), &DebugExpression::Integer(30), Some(frame))
        .expect("debugger value");
    stopped(session.step_into().expect("source initializer"));
    let locals = scope_reference(&mut session, "Locals");
    assert_eq!(
        named(
            &session.variables(locals, 0, 10).expect("overwritten").items,
            "Count"
        )
        .value,
        "1"
    );
}

#[test]
fn replacement_evaluates_once_and_failures_preserve_empty_storage() {
    let mut session = DebugSession::new(assignment_executable()).expect("debug session");
    let frame = session.stack(0, 1).expect("stack").items[0].id;
    let locals = scope_reference(&mut session, "Locals");
    let generation = locals;

    assert_eq!(
        session
            .set_expression(&root("Count"), &DebugExpression::Boolean(true), Some(frame))
            .expect_err("type")
            .kind,
        DebugErrorKind::VariableValueType
    );
    session.evaluation_cancel_handle().cancel();
    assert_eq!(
        session
            .set_expression(
                &root("Count"),
                &DebugExpression::Call {
                    callee: Box::new(DebugExpression::Callable("helper".to_string())),
                    arguments: vec![DebugExpression::Integer(1)],
                },
                Some(frame),
            )
            .expect_err("cancelled")
            .kind,
        DebugErrorKind::CallCancelled
    );
    assert_eq!(
        session
            .set_expression_with_limits(
                &root("Count"),
                &DebugExpression::Binary {
                    operation: DebugBinaryOperation::Add,
                    left: Box::new(DebugExpression::Integer(1)),
                    right: Box::new(DebugExpression::Integer(2)),
                },
                Some(frame),
                DebugEvaluationLimits {
                    max_operations: 1,
                    ..DebugEvaluationLimits::default()
                },
            )
            .expect_err("operation limit")
            .kind,
        DebugErrorKind::EvaluationLimit
    );
    assert!(session.scopes(frame).is_ok(), "failures preserve frames");
    assert_eq!(
        named(
            &session
                .variables(generation, 0, 10)
                .expect("preserved handles")
                .items,
            "Count"
        )
        .value,
        "<uninitialized>"
    );
}

#[test]
fn immutable_parameters_captures_and_descendants_are_rejected() {
    let mut session = DebugSession::new(assignment_executable()).expect("debug session");
    let frame = session.stack(0, 1).expect("stack").items[0].id;
    assert_eq!(
        session
            .set_expression(&root("Frozen"), &DebugExpression::Integer(1), Some(frame))
            .expect_err("immutable")
            .kind,
        DebugErrorKind::VariableNotMutable
    );
    assert_eq!(
        session
            .set_expression(
                &field("Nested", "X"),
                &DebugExpression::Integer(1),
                Some(frame)
            )
            .expect_err("field")
            .kind,
        DebugErrorKind::VariablePathUnsupported
    );
    assert_eq!(
        session
            .set_expression(
                &DebugAssignmentTarget {
                    root: "Items".to_string(),
                    selectors: vec![DebugAssignmentSelector::Index(DebugExpression::Integer(0))],
                },
                &DebugExpression::Integer(1),
                Some(frame),
            )
            .expect_err("index")
            .kind,
        DebugErrorKind::VariablePathUnsupported
    );
    assert_eq!(
        session
            .evaluate(&DebugExpression::Name("Count".to_string()), Some(frame))
            .expect_err("evaluate uninitialized")
            .kind,
        DebugErrorKind::UninitializedValue
    );
    let parameters = scope_reference(&mut session, "Parameters");
    assert_eq!(
        session
            .set_variable(parameters, "Arg", &DebugExpression::Integer(1))
            .expect_err("parameter")
            .kind,
        DebugErrorKind::VariableUnavailable
    );
    let captures = scope_reference(&mut session, "Captures");
    assert_eq!(
        session
            .set_variable(captures, "Captured", &DebugExpression::Integer(1))
            .expect_err("capture")
            .kind,
        DebugErrorKind::VariableUnavailable
    );
}

#[test]
fn successful_initialization_expires_handles_and_targets_the_selected_task() {
    let mut session = DebugSession::new(assignment_executable()).expect("debug session");
    let locals = scope_reference(&mut session, "Locals");
    session
        .set_variable(locals, "Count", &DebugExpression::Integer(9))
        .expect("initialize");
    assert_eq!(
        session
            .set_variable(locals, "Count", &DebugExpression::Integer(1))
            .expect_err("expired")
            .kind,
        DebugErrorKind::VariableTargetExpired
    );

    let mut tasks = DebugSession::new(task_assignment_executable()).expect("task session");
    tasks
        .set_breakpoint(SourceBreakpoint {
            source: "test.fpas".to_string(),
            line: 20,
            column: None,
        })
        .expect("child breakpoint");
    let stop = stopped(tasks.continue_execution().expect("run to child"));
    assert_eq!(stop.task_id, 1);
    let child = tasks.stack_for_task(1, 0, 1).expect("child stack").items[0].id;
    let main = tasks.stack_for_task(0, 0, 1).expect("main stack").items[0].id;
    tasks
        .set_expression(&root("Count"), &DebugExpression::Integer(5), Some(child))
        .expect("child initialization");
    assert_eq!(
        tasks.scopes(main).expect_err("main snapshot expired").kind,
        DebugErrorKind::UnknownFrame
    );
    let child = tasks.stack_for_task(1, 0, 1).expect("fresh child").items[0].id;
    let locals = tasks
        .scopes(child)
        .expect("child scopes")
        .into_iter()
        .find(|scope| scope.name == "Locals")
        .expect("child locals")
        .variables_reference;
    assert_eq!(
        named(
            &tasks.variables(locals, 0, 10).expect("child locals").items,
            "Count"
        )
        .value,
        "5"
    );
}
