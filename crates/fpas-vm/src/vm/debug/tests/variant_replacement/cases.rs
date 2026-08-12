//! Session-level constructor evaluation, commit, and failure coverage.

use super::*;

#[test]
fn qualified_enum_constructors_evaluate_fieldless_and_data_carrying_variants() {
    let mut session = DebugSession::new(variant_executable()).expect("debug session");
    stop_with_variants(&mut session);
    let frame = session.stack(0, 1).expect("stack").items[0].id;

    let empty = session
        .evaluate(&fieldless("Choice", "Empty"), Some(frame))
        .expect("fieldless constructor");
    assert_eq!(empty.value, "Choice.Empty");
    assert_eq!(empty.named_variables, 0);

    let count = session
        .evaluate(
            &enum_call("cHoIcE.cOuNt", vec![DebugExpression::Integer(4)]),
            Some(frame),
        )
        .expect("mixed-case constructor");
    assert_eq!(count.value, "Choice.Count");
    assert_eq!(count.named_variables, 1);

    let constructed = session
        .evaluate(&pair(1, 2), Some(frame))
        .expect("pair constructor");
    assert_eq!(constructed.value, "Choice.Pair");
    assert_eq!(constructed.named_variables, 2);
}

#[test]
fn constructor_arguments_evaluate_once_under_the_shared_operation_budget() {
    let mut session = DebugSession::new(variant_executable()).expect("debug session");
    stop_with_variants(&mut session);
    let frame = session.stack(0, 1).expect("stack").items[0].id;
    let expression = enum_call(
        "Choice.Pair",
        vec![
            DebugExpression::Binary {
                operation: DebugBinaryOperation::Add,
                left: Box::new(DebugExpression::Integer(1)),
                right: Box::new(DebugExpression::Integer(2)),
            },
            DebugExpression::Binary {
                operation: DebugBinaryOperation::Add,
                left: Box::new(DebugExpression::Integer(3)),
                right: Box::new(DebugExpression::Integer(4)),
            },
        ],
    );
    let limits = DebugEvaluationLimits {
        max_operations: 7,
        ..DebugEvaluationLimits::default()
    };
    let constructed = session
        .evaluate_with_limits(&expression, Some(frame), limits)
        .expect("exact argument budget");
    assert_eq!(constructed.value, "Choice.Pair");
    assert_eq!(
        session
            .evaluate_with_limits(
                &expression,
                Some(frame),
                DebugEvaluationLimits {
                    max_operations: 6,
                    ..DebugEvaluationLimits::default()
                },
            )
            .expect_err("short argument budget")
            .kind,
        DebugErrorKind::EvaluationLimit
    );
}

#[test]
fn complete_enum_result_and_option_replacements_commit_and_continue() {
    let mut session = DebugSession::new(variant_executable()).expect("debug session");
    stop_with_variants(&mut session);
    let locals = scope_reference(&mut session, "Locals");
    session
        .set_variable(locals, "Selected", &pair(10, 20))
        .expect("handle-based enum replacement");
    let frame = session.stack(0, 1).expect("fresh stack").items[0].id;
    session
        .set_expression(
            &root("EmptyValue"),
            &enum_call("Choice.Count", vec![DebugExpression::Integer(4)]),
            Some(frame),
        )
        .expect("fieldless to data-carrying");
    let frame = session.stack(0, 1).expect("fresh stack").items[0].id;
    session
        .set_expression(
            &root("Outcome"),
            &DebugExpression::ResultError(Box::new(DebugExpression::String("failed".to_string()))),
            Some(frame),
        )
        .expect("ok to error");
    let frame = session.stack(0, 1).expect("fresh stack").items[0].id;
    session
        .set_expression(&root("Optional"), &DebugExpression::OptionNone, Some(frame))
        .expect("some to none");
    let frame = session.stack(0, 1).expect("fresh stack").items[0].id;
    session
        .set_expression(
            &root("Missing"),
            &DebugExpression::OptionSome(Box::new(DebugExpression::Integer(8))),
            Some(frame),
        )
        .expect("none to some");
    let frame = session.stack(0, 1).expect("fresh stack").items[0].id;
    session
        .set_expression(
            &root("Outcome"),
            &DebugExpression::ResultOk(Box::new(DebugExpression::Integer(9))),
            Some(frame),
        )
        .expect("error to ok");

    let locals = scope_reference(&mut session, "Locals");
    let variables = session.variables(locals, 0, 20).expect("committed locals");
    assert_eq!(named(&variables.items, "Selected").value, "Choice.Pair");
    let selected = session
        .variables(
            named(&variables.items, "Selected").variables_reference,
            0,
            10,
        )
        .expect("pair fields");
    assert_eq!(named(&selected.items, "Left").value, "10");
    assert_eq!(named(&selected.items, "Right").value, "20");
    assert_eq!(named(&variables.items, "EmptyValue").value, "Choice.Count");
    assert_eq!(named(&variables.items, "Outcome").value, "Ok(...)");
    assert_eq!(named(&variables.items, "Optional").value, "None");
    assert_eq!(named(&variables.items, "Missing").value, "Some(...)");

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
    panic!("helper never received the replaced variant payload");
}

#[test]
fn nested_roots_and_copy_on_write_aliases_follow_existing_mutability_rules() {
    let mut session = DebugSession::new(variant_executable()).expect("debug session");
    stop_with_variants(&mut session);
    let frame = session.stack(0, 1).expect("stack").items[0].id;
    session
        .set_expression(&field("Holder", "Item"), &pair(1, 2), Some(frame))
        .expect("record field enum");
    let frame = session.stack(0, 1).expect("fresh stack").items[0].id;
    session
        .set_expression(
            &DebugAssignmentTarget {
                root: "Items".to_string(),
                selectors: vec![DebugAssignmentSelector::Index(DebugExpression::Integer(0))],
            },
            &fieldless("Choice", "Empty"),
            Some(frame),
        )
        .expect("array element enum");
    session
        .set_expression(&root("G"), &pair(3, 4), None)
        .expect("global enum");
    let frame = session.stack(0, 1).expect("fresh stack").items[0].id;
    session
        .set_expression(&root("Nested"), &DebugExpression::OptionNone, Some(frame))
        .expect("nested option");
    let frame = session.stack(0, 1).expect("fresh stack").items[0].id;
    session
        .set_expression(
            &root("Packed"),
            &DebugExpression::ResultError(Box::new(DebugExpression::String("fail".to_string()))),
            Some(frame),
        )
        .expect("nested result");
    let frame = session.stack(0, 1).expect("fresh stack").items[0].id;
    session
        .set_expression(&root("Selected"), &pair(10, 20), Some(frame))
        .expect("replace aliased root");

    let locals = scope_reference(&mut session, "Locals");
    let variables = session.variables(locals, 0, 20).expect("locals");
    assert_eq!(named(&variables.items, "Alias").value, "Choice.Count");
    let holder = session
        .variables(named(&variables.items, "Holder").variables_reference, 0, 10)
        .expect("holder fields");
    assert_eq!(named(&holder.items, "Item").value, "Choice.Pair");
    let items = session
        .variables(named(&variables.items, "Items").variables_reference, 0, 10)
        .expect("array items");
    assert_eq!(named(&items.items, "[0]").value, "Choice.Empty");
    assert_eq!(named(&variables.items, "Nested").value, "None");
    assert_eq!(named(&variables.items, "Packed").value, "Error(...)");
    let globals = scope_reference(&mut session, "Globals");
    assert_eq!(
        session.variables(globals, 0, 1).expect("globals").items[0].value,
        "Choice.Pair"
    );
}

#[test]
fn constructor_and_replacement_failures_are_atomic() {
    let mut session = DebugSession::new(variant_executable()).expect("debug session");
    stop_with_variants(&mut session);
    let frame = session.stack(0, 1).expect("stack").items[0].id;
    let locals = scope_reference(&mut session, "Locals");
    let variables = session.variables(locals, 0, 20).expect("locals");
    let selected = named(&variables.items, "Selected").variables_reference;

    assert_eq!(
        session
            .evaluate(
                &enum_call("Pair", vec![DebugExpression::Integer(1)]),
                Some(frame)
            )
            .expect_err("short name")
            .kind,
        DebugErrorKind::UnknownCallable
    );
    assert_eq!(
        session
            .evaluate(&enum_call("Missing.Empty", Vec::new()), Some(frame))
            .expect_err("unknown owner")
            .kind,
        DebugErrorKind::UnknownCallable
    );
    assert_eq!(
        session
            .evaluate(&enum_call("Choice.Missing", Vec::new()), Some(frame))
            .expect_err("unknown variant")
            .kind,
        DebugErrorKind::UnknownCallable
    );
    assert_eq!(
        session
            .evaluate(
                &enum_call("Choice.Pair", vec![DebugExpression::Integer(1)]),
                Some(frame)
            )
            .expect_err("wrong arity")
            .kind,
        DebugErrorKind::CallArity
    );
    assert_eq!(
        session
            .evaluate(
                &enum_call(
                    "Choice.Count",
                    vec![DebugExpression::String("wrong".to_string())]
                ),
                Some(frame)
            )
            .expect_err("wrong constructor argument type")
            .kind,
        DebugErrorKind::EvaluationType
    );
    assert_eq!(
        session
            .set_expression(&root("Selected"), &fieldless("Other", "Only"), Some(frame))
            .expect_err("wrong enum owner")
            .kind,
        DebugErrorKind::VariableValueType
    );
    assert_eq!(
        session
            .set_expression(&root("Selected"), &DebugExpression::Integer(1), Some(frame),)
            .expect_err("wrong type")
            .kind,
        DebugErrorKind::VariableValueType
    );
    assert_eq!(
        session
            .set_expression(&root("Fixed"), &pair(1, 2), Some(frame))
            .expect_err("immutable")
            .kind,
        DebugErrorKind::VariableNotMutable
    );
    assert_eq!(
        session
            .set_expression(&field("Uninit", "Left"), &pair(1, 2), Some(frame))
            .expect_err("uninitialized descendant")
            .kind,
        DebugErrorKind::VariablePathUnsupported
    );
    assert_eq!(
        session
            .set_expression_with_limits(
                &root("Selected"),
                &pair(1, 2),
                Some(frame),
                DebugEvaluationLimits {
                    max_calls: 0,
                    ..DebugEvaluationLimits::default()
                },
            )
            .expect_err("call limit")
            .kind,
        DebugErrorKind::CallLimit
    );
    session.evaluation_cancel_handle().cancel();
    assert_eq!(
        session
            .set_expression(&root("Selected"), &pair(1, 2), Some(frame))
            .expect_err("cancelled constructor")
            .kind,
        DebugErrorKind::CallCancelled
    );

    let locals = scope_reference(&mut session, "Locals");
    let variables = session.variables(locals, 0, 20).expect("unchanged locals");
    assert_eq!(named(&variables.items, "Selected").value, "Choice.Count");
    assert!(session.scopes(frame).is_ok(), "failures preserve frames");

    session
        .set_expression(&root("Selected"), &pair(10, 20), Some(frame))
        .expect("successful switch");
    assert_eq!(
        session
            .set_variable(selected, "Value", &DebugExpression::Integer(1))
            .expect_err("expired payload child")
            .kind,
        DebugErrorKind::VariableTargetExpired
    );
    let frame = session.stack(0, 1).expect("fresh stack").items[0].id;
    assert_eq!(
        session
            .set_expression(
                &field("Selected", "Value"),
                &DebugExpression::Integer(1),
                Some(frame),
            )
            .expect_err("stale field name")
            .kind,
        DebugErrorKind::VariableTargetUnknown
    );
}

#[test]
fn colliding_function_names_keep_routine_resolution() {
    let mut session = DebugSession::new(collision_executable()).expect("debug session");
    stop_with_variants(&mut session);
    let frame = session.stack(0, 1).expect("stack").items[0].id;
    let result = session
        .evaluate(&pair(1, 2), Some(frame))
        .expect("function wins over constructor");
    assert_eq!(result.value, "99");
    session
        .set_expression(&root("Selected"), &pair(1, 2), Some(frame))
        .expect_err("function result is not an enum");
}
