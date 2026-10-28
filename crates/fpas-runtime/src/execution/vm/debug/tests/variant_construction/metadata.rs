//! Metadata discovery coverage for enum, Result, and Option descriptors.

use super::*;

#[test]
fn discovery_returns_canonical_enum_and_wrapper_descriptors() {
    let mut session = DebugSession::new(variant_executable()).expect("debug session");
    stop_with_variants(&mut session);
    let frame = session.stack(0, 1).expect("stack").items[0].id;
    let locals = scope_reference(&mut session, "Locals");
    let before = session
        .variables(locals, 0, 20)
        .expect("locals before discovery");
    let selected = session
        .describe_variant(&root("Selected"), Some(frame))
        .expect("enum discovery");
    assert_eq!(selected.type_name, "Choice");
    assert_eq!(
        selected
            .variants
            .iter()
            .map(|variant| variant.name.as_str())
            .collect::<Vec<_>>(),
        ["Choice.Empty", "Choice.Count", "Choice.Pair"]
    );
    assert!(selected.variants[0].fields.is_empty());
    assert_eq!(selected.variants[1].fields[0].name, "Value");
    assert_eq!(selected.variants[1].fields[0].type_name, "Integer");
    assert_eq!(
        selected.variants[2]
            .fields
            .iter()
            .map(|field| field.name.as_str())
            .collect::<Vec<_>>(),
        ["Left", "Right"]
    );

    let outcome = session
        .describe_variant(&root("Outcome"), Some(frame))
        .expect("result discovery");
    assert_eq!(outcome.type_name, "Result");
    assert_eq!(
        outcome
            .variants
            .iter()
            .map(|variant| variant.name.as_str())
            .collect::<Vec<_>>(),
        ["Ok", "Error"]
    );
    assert_eq!(outcome.variants[0].fields[0].name, "value");
    assert_eq!(outcome.variants[0].fields[0].type_name, "Integer");
    assert_eq!(outcome.variants[1].fields[0].type_name, "String");

    let optional = session
        .describe_variant(&root("Optional"), Some(frame))
        .expect("option discovery");
    assert_eq!(optional.type_name, "Option");
    assert_eq!(
        optional
            .variants
            .iter()
            .map(|variant| variant.name.as_str())
            .collect::<Vec<_>>(),
        ["Some", "None"]
    );
    assert!(optional.variants[1].fields.is_empty());

    let uninitialized = session
        .describe_variant(&root("Uninit"), Some(frame))
        .expect("uninitialized root discovery");
    assert_eq!(uninitialized.type_name, "Choice");

    let after = session
        .variables(locals, 0, 20)
        .expect("locals after discovery");
    assert_eq!(
        named(&before.items, "Selected").variables_reference,
        named(&after.items, "Selected").variables_reference
    );
}

#[test]
fn discovery_rejects_non_wrapper_unknown_and_immutable_targets() {
    let mut session = DebugSession::new(variant_executable()).expect("debug session");
    stop_with_variants(&mut session);
    let frame = session.stack(0, 1).expect("stack").items[0].id;
    let record = session
        .describe_variant(&root("Holder"), Some(frame))
        .expect_err("record is not a wrapper");
    assert_eq!(record.kind, DebugErrorKind::VariablePathUnsupported);
    assert!(!record.hint.is_empty());

    let frozen = session
        .describe_variant(&root("Fixed"), Some(frame))
        .expect_err("immutable");
    assert_eq!(frozen.kind, DebugErrorKind::VariableNotMutable);

    let missing = session
        .describe_variant(&root("MissingName"), Some(frame))
        .expect_err("unknown");
    assert_eq!(missing.kind, DebugErrorKind::VariableTargetUnknown);
}

#[test]
fn discovery_clears_a_pending_evaluation_cancellation() {
    let mut session = DebugSession::new(order_executable()).expect("order session");
    stop_order_session(&mut session);
    let frame = session.stack(0, 1).expect("stack").items[0].id;
    session.evaluation_cancel_handle().cancel();
    session
        .describe_variant(&root("Selected"), Some(frame))
        .expect("description clears cancellation");
    session
        .construct_variant(
            &root("Selected"),
            "Choice.Pair",
            &fields(&[("Left", next_call()), ("Right", next_call())]),
            Some(frame),
        )
        .expect("later construction is not cancelled");
}
