use super::*;

fn scope_reference(session: &mut DebugSession, scope_name: &str) -> u64 {
    let frame = session.stack(0, 1).expect("stack").items[0].id;
    session
        .scopes(frame)
        .expect("scopes")
        .into_iter()
        .find(|scope| scope.name == scope_name)
        .expect("requested scope")
        .variables_reference
}

#[test]
fn mutable_global_updates_atomically_and_type_failure_preserves_snapshot() {
    let mut session = DebugSession::new(inspection_executable()).expect("debug session");
    stopped(session.step_into().expect("initialize values"));
    let globals = scope_reference(&mut session, "Globals");

    let wrong = session
        .set_variable(globals, "G", &DebugExpression::String("wrong".to_string()))
        .expect_err("type mismatch");
    assert_eq!(wrong.kind, DebugErrorKind::VariableValueType);
    assert_eq!(
        session
            .variables(globals, 0, 1)
            .expect("unchanged snapshot")
            .items[0]
            .value,
        "42"
    );

    let updated = session
        .set_variable(globals, "G", &DebugExpression::Integer(99))
        .expect("global mutation");
    assert_eq!(updated.value, "99");
    assert_eq!(
        session
            .set_variable(globals, "G", &DebugExpression::Integer(1))
            .expect_err("expired target")
            .kind,
        DebugErrorKind::VariableTargetExpired
    );
    let globals = scope_reference(&mut session, "Globals");
    assert_eq!(
        session
            .variables(globals, 0, 1)
            .expect("fresh globals")
            .items[0]
            .value,
        "99"
    );
}

#[test]
fn local_and_array_descendant_updates_are_observed_after_resume() {
    let mut session = DebugSession::new(inspection_executable()).expect("debug session");
    stopped(session.step_into().expect("initialize values"));
    let locals = scope_reference(&mut session, "Locals");
    let variables = session.variables(locals, 0, 10).expect("locals");
    let items = variables
        .items
        .iter()
        .find(|variable| variable.name == "Items")
        .expect("items")
        .variables_reference;

    session
        .set_variable(items, "[1]", &DebugExpression::Integer(9))
        .expect("array element mutation");
    let locals = scope_reference(&mut session, "Locals");
    let items = session
        .variables(locals, 0, 10)
        .expect("fresh locals")
        .items
        .into_iter()
        .find(|variable| variable.name == "Items")
        .expect("fresh items")
        .variables_reference;
    assert_eq!(
        session
            .variables(items, 0, 10)
            .expect("updated array")
            .items
            .iter()
            .map(|item| item.value.as_str())
            .collect::<Vec<_>>(),
        ["1", "9"]
    );

    let locals = scope_reference(&mut session, "Locals");
    session
        .set_variable(locals, "Answer", &DebugExpression::Integer(77))
        .expect("local mutation");
    stopped(session.step_into().expect("enter helper"));
    let parameters = scope_reference(&mut session, "Parameters");
    assert_eq!(
        session
            .variables(parameters, 0, 1)
            .expect("helper parameter")
            .items[0]
            .value,
        "77"
    );
    assert_eq!(
        session
            .set_variable(parameters, "Value", &DebugExpression::Integer(1))
            .expect_err("immutable parameter")
            .kind,
        DebugErrorKind::VariableNotMutable
    );
}

#[test]
fn failed_sessions_and_evaluation_only_children_are_not_mutable() {
    let mut failed = DebugSession::new(panic_executable()).expect("debug session");
    stopped(failed.step_into().expect("step to panic"));
    stopped(failed.continue_execution().expect("runtime failure"));
    assert_eq!(
        failed
            .set_variable(1, "x", &DebugExpression::Integer(1))
            .expect_err("failed state")
            .kind,
        DebugErrorKind::InvalidState
    );

    let mut session = DebugSession::new(inspection_executable()).expect("debug session");
    stopped(session.step_into().expect("initialize values"));
    let result = session
        .evaluate(
            &DebugExpression::Array(vec![DebugExpression::Integer(1)]),
            None,
        )
        .expect("evaluation array");
    assert_eq!(
        session
            .set_variable(
                result.variables_reference,
                "[0]",
                &DebugExpression::Integer(2),
            )
            .expect_err("evaluation-only child")
            .kind,
        DebugErrorKind::VariablePathUnsupported
    );
}
