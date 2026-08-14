//! Positive seeded empty-storage construction coverage.

use super::*;

#[test]
fn record_array_dictionary_and_payload_descendants_commit_from_an_explicit_seed() {
    let mut session = DebugSession::new(construction_executable()).expect("debug session");
    let frame = stop_with_empty(&mut session, "State");
    let result = session
        .initialize_storage(
            &nested("State", &["Nested", "X"]),
            &make_initial_state(),
            &DebugExpression::Integer(42),
            Some(frame),
        )
        .expect("nested field");
    assert_eq!(result.root, "State");
    assert_eq!(result.target, "State.Nested.X");
    assert_eq!(result.value.value, "42");
    assert_eq!(result.value.type_name, "integer");
    assert!(result.root_value.contains("Holder"));

    let frame = session.stack(0, 1).expect("fresh stack").items[0].id;
    session
        .initialize_storage(
            &index_target("Items", DebugExpression::Integer(1)),
            &DebugExpression::Array(vec![
                DebugExpression::Integer(1),
                DebugExpression::Integer(2),
                DebugExpression::Integer(3),
            ]),
            &DebugExpression::Integer(9),
            Some(frame),
        )
        .expect("array element");

    let frame = session.stack(0, 1).expect("fresh stack").items[0].id;
    let dictionary = session
        .initialize_storage(
            &index_target("Scores", DebugExpression::String("it's".to_string())),
            &DebugExpression::Dictionary(vec![(
                DebugExpression::String("it's".to_string()),
                DebugExpression::Integer(1),
            )]),
            &DebugExpression::Integer(8),
            Some(frame),
        )
        .expect("dictionary value");
    assert_eq!(dictionary.target, "Scores['it''s']");

    let frame = session.stack(0, 1).expect("fresh stack").items[0].id;
    session
        .initialize_storage(
            &nested("Selected", &["Value"]),
            &DebugExpression::Call {
                callee: Box::new(DebugExpression::Callable("Choice.Count".to_string())),
                arguments: vec![DebugExpression::Integer(5)],
            },
            &DebugExpression::Integer(11),
            Some(frame),
        )
        .expect("enum payload");

    let frame = session.stack(0, 1).expect("fresh stack").items[0].id;
    session
        .initialize_storage(
            &nested("Outcome", &["value"]),
            &DebugExpression::ResultOk(Box::new(DebugExpression::Integer(6))),
            &DebugExpression::Integer(12),
            Some(frame),
        )
        .expect("result payload");

    let frame = session.stack(0, 1).expect("fresh stack").items[0].id;
    session
        .initialize_storage(
            &nested("Optional", &["value"]),
            &DebugExpression::OptionSome(Box::new(DebugExpression::Integer(7))),
            &DebugExpression::Integer(13),
            Some(frame),
        )
        .expect("option payload");

    let locals = scope_reference(&mut session, "Locals");
    let variables = session.variables(locals, 0, 20).expect("locals");
    let items = session
        .variables(named(&variables.items, "Items").variables_reference, 0, 10)
        .expect("items");
    assert_eq!(named(&items.items, "[1]").value, "9");
    assert_eq!(named(&variables.items, "Selected").value, "Choice.Count");
    assert_eq!(named(&variables.items, "Outcome").value, "Ok(...)");
    assert_eq!(named(&variables.items, "Optional").value, "Some(...)");
}

#[test]
fn copied_seed_preserves_unmentioned_fields_and_global_roots_are_eligible() {
    let mut session = DebugSession::new(construction_executable()).expect("debug session");
    let _ = stop_with_initialized(&mut session, "Callback");
    let frame = session.stack(0, 1).expect("stack").items[0].id;
    assert_eq!(local_value(&mut session, "State"), "<uninitialized>");
    session
        .initialize_storage(
            &nested("State", &["Count"]),
            &DebugExpression::Name("GlobalState".to_string()),
            &DebugExpression::Integer(99),
            Some(frame),
        )
        .expect("copied global seed");
    let locals = scope_reference(&mut session, "Locals");
    let variables = session.variables(locals, 0, 20).expect("locals");
    let state = named(&variables.items, "State");
    let fields = session
        .variables(state.variables_reference, 0, 20)
        .expect("state fields");
    assert_eq!(named(&fields.items, "Count").value, "99");
    assert!(named(&fields.items, "Nested").value.contains("Point"));

    let mut global = DebugSession::new(construction_executable()).expect("global session");
    let _ = stop_with_empty(&mut global, "State");
    global
        .initialize_storage(
            &nested("GlobalState", &["Count"]),
            &make_initial_state(),
            &DebugExpression::Integer(4),
            None,
        )
        .expect("empty global");
    let globals = scope_reference(&mut global, "Globals");
    assert!(
        named(
            &global.variables(globals, 0, 10).expect("globals").items,
            "GlobalState"
        )
        .value
        .contains("Holder")
    );
}

#[test]
fn initializer_indexes_and_replacement_evaluate_once_under_one_budget() {
    let mut session = DebugSession::new(construction_executable()).expect("debug session");
    let _ = stop_with_initialized(&mut session, "Callback");
    let frame = session.stack(0, 1).expect("stack").items[0].id;
    assert_eq!(local_value(&mut session, "Items"), "<uninitialized>");
    session
        .initialize_storage(
            &index_target("Items", next_call()),
            &DebugExpression::Array(vec![
                next_call(),
                DebugExpression::Integer(0),
                DebugExpression::Integer(0),
            ]),
            &next_call(),
            Some(frame),
        )
        .expect("ordered evaluation");
    let locals = scope_reference(&mut session, "Locals");
    let variables = session.variables(locals, 0, 10).expect("locals");
    let items = session
        .variables(named(&variables.items, "Items").variables_reference, 0, 10)
        .expect("items");
    assert_eq!(named(&items.items, "[0]").value, "1");
    assert_eq!(named(&items.items, "[1]").value, "0");
    assert_eq!(named(&items.items, "[2]").value, "3");
}

#[test]
fn successful_commit_expires_handles_and_source_initializer_overwrites() {
    let mut session = DebugSession::new(construction_executable()).expect("debug session");
    let frame = stop_with_empty(&mut session, "State");
    let stale = scope_reference(&mut session, "Locals");
    session
        .initialize_storage(
            &nested("State", &["Count"]),
            &make_initial_state(),
            &DebugExpression::Integer(42),
            Some(frame),
        )
        .expect("initialize");
    assert_eq!(
        session
            .variables(stale, 0, 10)
            .expect_err("expired handles")
            .kind,
        DebugErrorKind::UnknownVariablesReference
    );
    assert!(local_value(&mut session, "State").contains("Holder"));
    let _ = session.continue_execution().expect("continue");
    let output = session.output().lines.join("\n");
    assert!(
        output.contains("1"),
        "source initializer overwrites debugger Count: {output:?}"
    );
}
