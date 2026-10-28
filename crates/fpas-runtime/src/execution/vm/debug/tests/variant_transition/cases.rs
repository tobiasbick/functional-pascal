//! Session-level qualified variant-transition commit and failure coverage.

use super::*;

#[test]
fn inactive_single_field_enum_option_and_result_transitions_commit() {
    let mut session = DebugSession::new(variant_executable()).expect("debug session");
    stop_with_variants(&mut session);
    let frame = session.stack(0, 1).expect("stack").items[0].id;
    session
        .set_expression(
            &qualified("EmptyValue", &["Count", "Value"]),
            &DebugExpression::Integer(4),
            Some(frame),
        )
        .expect("empty to count");
    let frame = session.stack(0, 1).expect("fresh stack").items[0].id;
    session
        .set_expression(
            &qualified("Missing", &["Some", "value"]),
            &DebugExpression::Integer(8),
            Some(frame),
        )
        .expect("none to some");
    let frame = session.stack(0, 1).expect("fresh stack").items[0].id;
    session
        .set_expression(
            &qualified("Outcome", &["Error", "value"]),
            &DebugExpression::String("failed".to_string()),
            Some(frame),
        )
        .expect("ok to error");
    let frame = session.stack(0, 1).expect("fresh stack").items[0].id;
    session
        .set_expression(
            &qualified("Outcome", &["Ok", "value"]),
            &DebugExpression::Integer(9),
            Some(frame),
        )
        .expect("error to ok");

    let locals = scope_reference(&mut session, "Locals");
    let variables = session.variables(locals, 0, 20).expect("committed locals");
    assert_eq!(named(&variables.items, "EmptyValue").value, "Choice.Count");
    let count = session
        .variables(
            named(&variables.items, "EmptyValue").variables_reference,
            0,
            10,
        )
        .expect("count field");
    assert_eq!(named(&count.items, "Value").value, "4");
    assert_eq!(named(&variables.items, "Missing").value, "Some(...)");
    assert_eq!(named(&variables.items, "Outcome").value, "Ok(...)");
}

#[test]
fn active_qualified_targets_normalize_to_payload_replacement() {
    let mut session = DebugSession::new(variant_executable()).expect("debug session");
    stop_with_variants(&mut session);
    let frame = session.stack(0, 1).expect("stack").items[0].id;
    session
        .set_expression(
            &qualified("Selected", &["cOuNt", "vAlUe"]),
            &DebugExpression::Integer(10),
            Some(frame),
        )
        .expect("active qualified enum");
    let frame = session.stack(0, 1).expect("fresh stack").items[0].id;
    session
        .set_expression(
            &qualified("Optional", &["Some", "value"]),
            &DebugExpression::Integer(70),
            Some(frame),
        )
        .expect("active qualified option");
    let frame = session.stack(0, 1).expect("fresh stack").items[0].id;
    session
        .set_expression(
            &qualified("Outcome", &["Ok", "value"]),
            &DebugExpression::Integer(20),
            Some(frame),
        )
        .expect("active qualified result");

    let locals = scope_reference(&mut session, "Locals");
    let variables = session.variables(locals, 0, 20).expect("locals");
    let selected = session
        .variables(
            named(&variables.items, "Selected").variables_reference,
            0,
            10,
        )
        .expect("selected fields");
    assert_eq!(named(&selected.items, "Value").value, "10");
    assert_eq!(named(&variables.items, "Alias").value, "Choice.Count");
    let alias = session
        .variables(named(&variables.items, "Alias").variables_reference, 0, 10)
        .expect("aliased fields");
    assert_eq!(named(&alias.items, "Value").value, "1");
    let optional = session
        .variables(
            named(&variables.items, "Optional").variables_reference,
            0,
            10,
        )
        .expect("optional fields");
    assert_eq!(named(&optional.items, "value").value, "70");
    let outcome = session
        .variables(
            named(&variables.items, "Outcome").variables_reference,
            0,
            10,
        )
        .expect("outcome fields");
    assert_eq!(named(&outcome.items, "value").value, "20");
}

#[test]
fn nested_prefixes_replace_only_the_selected_wrapper() {
    let mut session = DebugSession::new(variant_executable()).expect("debug session");
    stop_with_variants(&mut session);
    let frame = session.stack(0, 1).expect("stack").items[0].id;
    session
        .set_expression(
            &field("Holder", "Item"),
            &fieldless("Choice", "Empty"),
            Some(frame),
        )
        .expect("reset nested enum");
    let frame = session.stack(0, 1).expect("fresh stack").items[0].id;
    session
        .set_expression(
            &qualified("Holder", &["Item", "Count", "Value"]),
            &DebugExpression::Integer(5),
            Some(frame),
        )
        .expect("nested record transition");
    let frame = session.stack(0, 1).expect("fresh stack").items[0].id;
    session
        .set_expression(
            &DebugAssignmentTarget {
                root: "Items".to_string(),
                selectors: vec![
                    DebugAssignmentSelector::Index(DebugExpression::Integer(0)),
                    DebugAssignmentSelector::Field("Count".to_string()),
                    DebugAssignmentSelector::Field("Value".to_string()),
                ],
            },
            &DebugExpression::Integer(1),
            Some(frame),
        )
        .expect("array element inactive transition");
    session
        .set_expression(
            &qualified("G", &["Count", "Value"]),
            &DebugExpression::Integer(16),
            None,
        )
        .expect("global active qualified");
    let frame = session.stack(0, 1).expect("fresh stack").items[0].id;
    session
        .set_expression(&root("Nested"), &DebugExpression::OptionNone, Some(frame))
        .expect("reset nested option");
    let frame = session.stack(0, 1).expect("fresh stack").items[0].id;
    session
        .set_expression(
            &qualified("Nested", &["Some", "value"]),
            &DebugExpression::ResultError(Box::new(DebugExpression::String("inner".to_string()))),
            Some(frame),
        )
        .expect("nested option transition");

    let locals = scope_reference(&mut session, "Locals");
    let variables = session.variables(locals, 0, 20).expect("locals");
    let holder = session
        .variables(named(&variables.items, "Holder").variables_reference, 0, 10)
        .expect("holder");
    assert_eq!(named(&holder.items, "Item").value, "Choice.Count");
    let items = session
        .variables(named(&variables.items, "Items").variables_reference, 0, 10)
        .expect("items");
    assert_eq!(named(&items.items, "[0]").value, "Choice.Count");
    assert_eq!(named(&variables.items, "Nested").value, "Some(...)");
    let globals = scope_reference(&mut session, "Globals");
    assert_eq!(
        session.variables(globals, 0, 1).expect("globals").items[0].value,
        "Choice.Count"
    );
}

#[test]
fn replacement_evaluates_once_under_shared_limits_and_cancel() {
    let mut session = DebugSession::new(variant_executable()).expect("debug session");
    stop_with_variants(&mut session);
    let frame = session.stack(0, 1).expect("stack").items[0].id;
    let expression = DebugExpression::Binary {
        operation: DebugBinaryOperation::Add,
        left: Box::new(DebugExpression::Integer(1)),
        right: Box::new(DebugExpression::Integer(2)),
    };
    let limits = DebugEvaluationLimits {
        max_operations: 3,
        ..DebugEvaluationLimits::default()
    };
    session
        .set_expression_with_limits(
            &qualified("EmptyValue", &["Count", "Value"]),
            &expression,
            Some(frame),
            limits,
        )
        .expect("exact payload budget");
    let frame = session.stack(0, 1).expect("fresh stack").items[0].id;
    assert_eq!(
        session
            .set_expression_with_limits(
                &qualified("Missing", &["Some", "value"]),
                &expression,
                Some(frame),
                DebugEvaluationLimits {
                    max_operations: 2,
                    ..DebugEvaluationLimits::default()
                },
            )
            .expect_err("short payload budget")
            .kind,
        DebugErrorKind::EvaluationLimit
    );
    let frame = session.stack(0, 1).expect("fresh stack").items[0].id;
    session.evaluation_cancel_handle().cancel();
    assert_eq!(
        session
            .set_expression(
                &qualified("Missing", &["Some", "value"]),
                &enum_call("helper", vec![DebugExpression::Integer(1)]),
                Some(frame),
            )
            .expect_err("cancelled payload")
            .kind,
        DebugErrorKind::CallCancelled
    );
    let locals = scope_reference(&mut session, "Locals");
    let variables = session.variables(locals, 0, 20).expect("unchanged");
    assert_eq!(named(&variables.items, "Missing").value, "None");
}

#[test]
fn transition_failures_are_atomic_and_preserve_handles() {
    let mut session = DebugSession::new(variant_executable()).expect("debug session");
    stop_with_variants(&mut session);
    let frame = session.stack(0, 1).expect("stack").items[0].id;
    let locals = scope_reference(&mut session, "Locals");
    let variables = session.variables(locals, 0, 20).expect("locals");
    let selected = named(&variables.items, "Selected").variables_reference;

    let wrong_type = session
        .set_expression(
            &qualified("EmptyValue", &["Count", "Value"]),
            &DebugExpression::String("wrong".to_string()),
            Some(frame),
        )
        .expect_err("wrong payload type");
    assert_eq!(wrong_type.kind, DebugErrorKind::VariableValueType);
    assert!(!wrong_type.hint.is_empty());

    let unknown_variant = session
        .set_expression(
            &qualified("Selected", &["Missing", "Value"]),
            &DebugExpression::Integer(1),
            Some(frame),
        )
        .expect_err("unknown variant");
    assert_eq!(unknown_variant.kind, DebugErrorKind::VariableTargetUnknown);
    assert!(
        unknown_variant.hint.contains("Count.Value") || unknown_variant.hint.contains("Choice")
    );

    let invalid_target_precedes_replacement = session
        .set_expression(
            &qualified("Selected", &["Missing", "Value"]),
            &DebugExpression::Name("DoesNotExist".to_string()),
            Some(frame),
        )
        .expect_err("target error before replacement evaluation");
    assert_eq!(
        invalid_target_precedes_replacement.kind,
        DebugErrorKind::VariableTargetUnknown
    );

    let unknown_payload = session
        .set_expression(
            &qualified("EmptyValue", &["Count", "Nope"]),
            &DebugExpression::Integer(1),
            Some(frame),
        )
        .expect_err("unknown payload name");
    assert_eq!(unknown_payload.kind, DebugErrorKind::VariableTargetUnknown);

    let unqualified = session
        .set_expression(
            &field("EmptyValue", "Value"),
            &DebugExpression::Integer(1),
            Some(frame),
        )
        .expect_err("unqualified inactive field");
    assert_eq!(unqualified.kind, DebugErrorKind::VariablePathUnsupported);
    assert!(
        unqualified.hint.contains("Count.Value") || unqualified.hint.contains("Choice.Count"),
        "{}",
        unqualified.hint
    );

    let fieldless = session
        .set_expression(
            &qualified("Selected", &["Empty"]),
            &DebugExpression::Integer(1),
            Some(frame),
        )
        .expect_err("fieldless descendant");
    assert_eq!(fieldless.kind, DebugErrorKind::VariablePathUnsupported);
    assert!(
        fieldless.hint.contains("Choice.Empty"),
        "{}",
        fieldless.hint
    );

    let multi_field = session
        .set_expression(
            &qualified("Selected", &["Pair", "Left"]),
            &DebugExpression::Integer(1),
            Some(frame),
        )
        .expect_err("multi-field descendant");
    assert_eq!(multi_field.kind, DebugErrorKind::VariablePathUnsupported);
    assert!(
        multi_field.hint.contains("Choice.Pair"),
        "{}",
        multi_field.hint
    );

    let uninitialized = session
        .set_expression(
            &qualified("Uninit", &["Count", "Value"]),
            &DebugExpression::Integer(1),
            Some(frame),
        )
        .expect_err("uninitialized root");
    assert_eq!(uninitialized.kind, DebugErrorKind::VariablePathUnsupported);

    let immutable = session
        .set_expression(
            &qualified("Fixed", &["Count", "Value"]),
            &DebugExpression::Integer(1),
            Some(frame),
        )
        .expect_err("immutable");
    assert_eq!(immutable.kind, DebugErrorKind::VariableNotMutable);

    let locals = scope_reference(&mut session, "Locals");
    let variables = session.variables(locals, 0, 20).expect("unchanged locals");
    assert_eq!(named(&variables.items, "EmptyValue").value, "Choice.Empty");
    assert_eq!(named(&variables.items, "Selected").value, "Choice.Count");
    assert!(session.scopes(frame).is_ok(), "failures preserve frames");

    session
        .set_expression(
            &qualified("EmptyValue", &["Count", "Value"]),
            &DebugExpression::Integer(4),
            Some(frame),
        )
        .expect("successful transition");
    assert_eq!(
        session
            .set_variable(selected, "Value", &DebugExpression::Integer(1))
            .expect_err("expired payload child")
            .kind,
        DebugErrorKind::VariableTargetExpired
    );
}

#[test]
fn continuation_observes_the_new_target_variant() {
    let mut session = DebugSession::new(variant_executable()).expect("debug session");
    stop_with_variants(&mut session);
    let frame = session.stack(0, 1).expect("stack").items[0].id;
    session
        .set_expression(
            &qualified("Selected", &["Count", "Value"]),
            &DebugExpression::Integer(10),
            Some(frame),
        )
        .expect("active payload for helper");
    stopped(session.step_into().expect("load mutated field"));
    for _ in 0..8 {
        if let Ok(stack) = session.stack(0, 1)
            && let Ok(scopes) = session.scopes(stack.items[0].id)
            && scopes.iter().any(|scope| scope.name == "Parameters")
        {
            let parameters = scope_reference(&mut session, "Parameters");
            assert_eq!(
                session
                    .variables(parameters, 0, 1)
                    .expect("helper parameter")
                    .items[0]
                    .value,
                "10"
            );
            return;
        }
        let _ = stopped(session.step_into().expect("enter helper"));
    }
    panic!("helper never received the transitioned payload");
}
