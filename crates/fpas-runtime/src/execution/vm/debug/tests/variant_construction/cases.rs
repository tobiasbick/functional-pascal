//! Session-level variant construction, ordering, storage, and rollback coverage.

use super::*;

#[test]
fn fieldless_and_wrapper_construction_commits_complete_values() {
    let mut session = DebugSession::new(variant_executable()).expect("debug session");
    stop_with_variants(&mut session);
    let frame = session.stack(0, 1).expect("stack").items[0].id;
    let stale = frame;
    let empty = session
        .construct_variant(&root("Selected"), "Choice.Empty", &[], Some(stale))
        .expect("fieldless enum");
    assert_eq!(empty.variant, "Choice.Empty");
    assert_eq!(empty.value.value, "Choice.Empty");
    let expired = session
        .construct_variant(&root("Selected"), "Choice.Empty", &[], Some(stale))
        .expect_err("expired frame");
    assert_eq!(expired.kind, DebugErrorKind::UnknownFrame);

    let frame = session.stack(0, 1).expect("fresh stack").items[0].id;
    session
        .construct_variant(&root("Missing"), "None", &[], Some(frame))
        .expect("option none");
    let frame = session.stack(0, 1).expect("fresh stack").items[0].id;
    session
        .construct_variant(
            &root("Optional"),
            "some",
            &fields(&[("value", DebugExpression::Integer(8))]),
            Some(frame),
        )
        .expect("option some");
    let frame = session.stack(0, 1).expect("fresh stack").items[0].id;
    session
        .construct_variant(
            &root("Outcome"),
            "Error",
            &fields(&[("value", DebugExpression::String("failed".to_string()))]),
            Some(frame),
        )
        .expect("result error");
    let frame = session.stack(0, 1).expect("fresh stack").items[0].id;
    session
        .construct_variant(
            &root("Outcome"),
            "Ok",
            &fields(&[("value", DebugExpression::Integer(9))]),
            Some(frame),
        )
        .expect("result ok");

    let locals = scope_reference(&mut session, "Locals");
    let variables = session.variables(locals, 0, 20).expect("locals");
    assert_eq!(named(&variables.items, "Selected").value, "Choice.Empty");
    assert_eq!(named(&variables.items, "Missing").value, "None");
    assert_eq!(named(&variables.items, "Optional").value, "Some(...)");
    assert_eq!(named(&variables.items, "Outcome").value, "Ok(...)");
}

#[test]
fn multi_field_construction_evaluates_declaration_order() {
    let mut session = DebugSession::new(order_executable()).expect("order session");
    stop_order_session(&mut session);
    let frame = session.stack(0, 1).expect("stack").items[0].id;
    let constructed = session
        .construct_variant(
            &root("Selected"),
            "Choice.Pair",
            &fields(&[("Right", next_call()), ("Left", next_call())]),
            Some(frame),
        )
        .expect("ordered pair");
    assert_eq!(constructed.variant, "Choice.Pair");
    let locals = scope_reference(&mut session, "Locals");
    let variables = session.variables(locals, 0, 10).expect("locals");
    let pair = session
        .variables(
            named(&variables.items, "Selected").variables_reference,
            0,
            10,
        )
        .expect("pair fields");
    assert_eq!(named(&pair.items, "Left").value, "1");
    assert_eq!(named(&pair.items, "Right").value, "2");
}

#[test]
fn nested_and_uninitialized_targets_construct_while_descendants_reject() {
    let mut session = DebugSession::new(variant_executable()).expect("debug session");
    stop_with_variants(&mut session);
    let frame = session.stack(0, 1).expect("stack").items[0].id;
    session
        .construct_variant(
            &field("Holder", "Item"),
            "Choice.Count",
            &fields(&[("Value", DebugExpression::Integer(4))]),
            Some(frame),
        )
        .expect("nested record field");
    let frame = session.stack(0, 1).expect("fresh stack").items[0].id;
    session
        .construct_variant(&index_target("Items", 0), "Choice.Empty", &[], Some(frame))
        .expect("array element");
    let frame = session.stack(0, 1).expect("fresh stack").items[0].id;
    session
        .construct_variant(&root("Uninit"), "Choice.Empty", &[], Some(frame))
        .expect("uninitialized root");

    let locals = scope_reference(&mut session, "Locals");
    let variables = session.variables(locals, 0, 20).expect("locals");
    assert_eq!(named(&variables.items, "Uninit").value, "Choice.Empty");
    let holder = session
        .variables(named(&variables.items, "Holder").variables_reference, 0, 10)
        .expect("holder");
    assert_eq!(named(&holder.items, "Item").value, "Choice.Count");

    let mut empty = DebugSession::new(variant_executable()).expect("entry session");
    let frame = empty.stack(0, 1).expect("entry").items[0].id;
    let nested = empty
        .construct_variant(&field("Holder", "Item"), "Choice.Empty", &[], Some(frame))
        .expect_err("uninitialized descendant");
    assert_eq!(nested.kind, DebugErrorKind::VariablePathUnsupported);
}

#[test]
fn exact_field_set_rejects_before_evaluation() {
    let mut session = DebugSession::new(order_executable()).expect("order session");
    stop_order_session(&mut session);
    let frame = session.stack(0, 1).expect("stack").items[0].id;
    let unknown = session
        .construct_variant(&root("Selected"), "Choice.Nope", &[], Some(frame))
        .expect_err("unknown variant");
    assert_eq!(unknown.kind, DebugErrorKind::VariantUnknown);
    assert!(unknown.hint.contains("Choice."));

    let missing = session
        .construct_variant(
            &root("Selected"),
            "Choice.Pair",
            &fields(&[("Left", next_call())]),
            Some(frame),
        )
        .expect_err("missing field");
    assert_eq!(missing.kind, DebugErrorKind::VariantFieldSet);

    let extra = session
        .construct_variant(
            &root("Selected"),
            "Choice.Pair",
            &fields(&[
                ("Left", DebugExpression::Integer(1)),
                ("Right", DebugExpression::Integer(2)),
                ("Z", DebugExpression::Integer(3)),
            ]),
            Some(frame),
        )
        .expect_err("extra field");
    assert_eq!(extra.kind, DebugErrorKind::VariantFieldSet);

    let duplicate = session
        .construct_variant(
            &root("Selected"),
            "Choice.Pair",
            &fields(&[
                ("Left", next_call()),
                ("left", next_call()),
                ("Right", DebugExpression::Integer(9)),
            ]),
            Some(frame),
        )
        .expect_err("duplicate field");
    assert_eq!(duplicate.kind, DebugErrorKind::VariantFieldSet);

    let locals = scope_reference(&mut session, "Locals");
    let variables = session.variables(locals, 0, 10).expect("unchanged");
    assert_eq!(named(&variables.items, "Selected").value, "Choice.Empty");
}

#[test]
fn construction_failures_preserve_the_original_value() {
    let mut session = DebugSession::new(variant_executable()).expect("debug session");
    stop_with_variants(&mut session);
    let frame = session.stack(0, 1).expect("stack").items[0].id;
    let locals = scope_reference(&mut session, "Locals");
    let before = named(
        &session.variables(locals, 0, 20).expect("before").items,
        "Selected",
    )
    .value
    .clone();

    let type_error = session
        .construct_variant(
            &root("Selected"),
            "Choice.Count",
            &fields(&[("Value", DebugExpression::String("nope".to_string()))]),
            Some(frame),
        )
        .expect_err("type");
    assert_eq!(type_error.kind, DebugErrorKind::VariableValueType);

    let frozen = session
        .construct_variant(&root("Fixed"), "Choice.Empty", &[], Some(frame))
        .expect_err("immutable");
    assert_eq!(frozen.kind, DebugErrorKind::VariableNotMutable);

    let effect = session
        .construct_variant(
            &root("Selected"),
            "Choice.Count",
            &fields(&[(
                "Value",
                DebugExpression::Call {
                    callee: Box::new(DebugExpression::Callable("WriteLn".to_string())),
                    arguments: vec![DebugExpression::Integer(1)],
                },
            )]),
            Some(frame),
        )
        .expect_err("effect");
    assert_eq!(effect.kind, DebugErrorKind::ForbiddenCallEffect);

    let limited = session
        .construct_variant_with_limits(
            &root("Selected"),
            "Choice.Count",
            &fields(&[(
                "Value",
                DebugExpression::Call {
                    callee: Box::new(DebugExpression::Callable("helper".to_string())),
                    arguments: vec![DebugExpression::Integer(1)],
                },
            )]),
            Some(frame),
            DebugEvaluationLimits {
                max_calls: 0,
                ..DebugEvaluationLimits::default()
            },
        )
        .expect_err("limit");
    assert!(matches!(
        limited.kind,
        DebugErrorKind::CallLimit | DebugErrorKind::EvaluationLimit
    ));

    let locals = scope_reference(&mut session, "Locals");
    let after = named(
        &session.variables(locals, 0, 20).expect("after").items,
        "Selected",
    )
    .value
    .clone();
    assert_eq!(before, after);
}
