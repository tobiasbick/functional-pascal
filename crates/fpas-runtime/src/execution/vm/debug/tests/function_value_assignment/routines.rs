//! Session-level assignment from executable routine names.

use super::*;

fn call(name: &str, value: i64) -> DebugExpression {
    DebugExpression::Call {
        callee: Box::new(super::name(name)),
        arguments: vec![DebugExpression::Integer(value)],
    }
}

#[test]
fn direct_routine_names_assign_when_no_binding_exists() {
    let mut session = DebugSession::new(assignment_executable()).expect("debug session");
    stop_with_functions(&mut session);
    let frame = session.stack(0, 1).expect("stack").items[0].id;
    let updated = session
        .set_expression(&root("Current"), &name("add_two"), Some(frame))
        .expect("simple routine");
    assert_eq!(updated.value, "<function add_two>");
    let frame = session.stack(0, 1).expect("fresh").items[0].id;
    assert_eq!(
        session
            .evaluate(&call("Current", 1), Some(frame))
            .expect("invoke add_two")
            .value,
        "3"
    );
    let locals = scope_reference(&mut session, "Locals");
    session
        .set_variable(locals, "Current", &name("ADD_ONE"))
        .expect("mixed-case short name");
    let frame = session.stack(0, 1).expect("after mixed").items[0].id;
    assert_eq!(
        session
            .evaluate(&call("Current", 1), Some(frame))
            .expect("invoke add_one")
            .value,
        "2"
    );
    session
        .set_expression(
            &root("Current"),
            &DebugExpression::Field {
                base: Box::new(name("Math")),
                name: "Transform".to_string(),
            },
            Some(frame),
        )
        .expect("qualified routine");
    let frame = session.stack(0, 1).expect("after qualified").items[0].id;
    assert_eq!(
        session
            .evaluate(&call("Current", 1), Some(frame))
            .expect("invoke transform")
            .value,
        "4"
    );
    let ambiguous = session
        .set_expression(&root("Current"), &name("Transform"), Some(frame))
        .expect_err("ambiguous short name");
    assert_eq!(ambiguous.kind, DebugErrorKind::AmbiguousCallable);
    assert_eq!(
        session
            .evaluate(&call("Current", 1), Some(frame))
            .expect("preserved after ambiguity")
            .value,
        "4"
    );
}

#[test]
fn visible_binding_shadows_an_equal_routine_name() {
    let mut session = DebugSession::new(assignment_executable()).expect("debug session");
    stop_with_functions(&mut session);
    let frame = session.stack(0, 1).expect("stack").items[0].id;
    session
        .set_expression(&root("Current"), &name("Backup"), Some(frame))
        .expect("binding wins");
    let frame = session.stack(0, 1).expect("fresh").items[0].id;
    assert_eq!(
        session
            .evaluate(&call("Current", 1), Some(frame))
            .expect("local Backup is add_two, not catalog backup")
            .value,
        "3"
    );
}

#[test]
fn capturing_routine_and_signature_mismatch_preserve_the_original_value() {
    let mut session = DebugSession::new(assignment_executable()).expect("debug session");
    stop_with_functions(&mut session);
    let frame = session.stack(0, 1).expect("stack").items[0].id;
    let locals = scope_reference(&mut session, "Locals");
    let capturing = session
        .set_expression(&root("Current"), &name("adder"), Some(frame))
        .expect_err("captures");
    assert_eq!(capturing.kind, DebugErrorKind::VariableValueType);
    assert!(capturing.message.contains("capture"), "{capturing:?}");
    let procedure = session
        .set_expression(&root("Current"), &name("helper"), Some(frame))
        .expect_err("signature");
    assert_eq!(procedure.kind, DebugErrorKind::VariableValueType);
    assert_eq!(
        named(
            &session.variables(locals, 0, 30).expect("preserved").items,
            "Current"
        )
        .value,
        "<function add_one>"
    );
}

#[test]
fn uninitialized_root_accepts_a_complete_routine_value() {
    let mut session = DebugSession::new(assignment_executable()).expect("debug session");
    stop_before_current(&mut session);
    let locals = scope_reference(&mut session, "Locals");
    assert_eq!(
        named(
            &session.variables(locals, 0, 30).expect("locals").items,
            "Current"
        )
        .value,
        "<uninitialized>"
    );
    assert_eq!(
        session
            .set_variable(locals, "Current", &name("add_two"))
            .expect("init from routine")
            .value,
        "<function add_two>"
    );
}

#[test]
fn qualified_routine_assignment_shares_selector_and_source_operation_budget() {
    let mut session = DebugSession::new(assignment_executable()).expect("debug session");
    stop_with_functions(&mut session);
    let target = DebugAssignmentTarget {
        root: "Items".to_string(),
        selectors: vec![DebugAssignmentSelector::Index(DebugExpression::Integer(0))],
    };
    let qualified = DebugExpression::Field {
        base: Box::new(name("Math")),
        name: "Transform".to_string(),
    };
    let frame = session.stack(0, 1).expect("stack").items[0].id;
    let exhausted = session
        .set_expression_with_limits(
            &target,
            &qualified,
            Some(frame),
            DebugEvaluationLimits {
                max_operations: 1,
                ..DebugEvaluationLimits::default()
            },
        )
        .expect_err("selector and routine source require two operations");
    assert_eq!(exhausted.kind, DebugErrorKind::EvaluationLimit);

    session
        .set_expression_with_limits(
            &target,
            &qualified,
            Some(frame),
            DebugEvaluationLimits {
                max_operations: 2,
                ..DebugEvaluationLimits::default()
            },
        )
        .expect("shared two-operation budget");
    let frame = session.stack(0, 1).expect("fresh stack").items[0].id;
    let selected = DebugExpression::Index {
        base: Box::new(name("Items")),
        index: Box::new(DebugExpression::Integer(0)),
    };
    assert_eq!(
        session
            .evaluate(
                &DebugExpression::Call {
                    callee: Box::new(selected),
                    arguments: vec![DebugExpression::Integer(1)],
                },
                Some(frame),
            )
            .expect("invoke assigned routine")
            .value,
        "4"
    );
}
