//! Session-level function-value assignment coverage.

use super::*;
use fpas_bytecode::Value;

fn call(name: &str, value: i64) -> DebugExpression {
    DebugExpression::Call {
        callee: Box::new(super::name(name)),
        arguments: vec![DebugExpression::Integer(value)],
    }
}

fn runtime(session: &DebugSession, expression: &DebugExpression, frame: u64) -> Value {
    session
        .evaluate_runtime_value(expression, Some(frame), DebugEvaluationLimits::default())
        .expect("runtime value")
}

#[test]
fn non_capturing_function_replaces_compatible_mutable_local() {
    let mut session = DebugSession::new(assignment_executable()).expect("debug session");
    stop_with_functions(&mut session);
    let locals = scope_reference(&mut session, "Locals");
    let updated = session
        .set_variable(locals, "Current", &name("Backup"))
        .expect("copy Backup");
    assert_eq!(updated.value, "<function add_two>");
    let frame = session.stack(0, 1).expect("fresh stack").items[0].id;
    assert_eq!(
        session
            .evaluate(&call("Current", 1), Some(frame))
            .expect("invoke")
            .value,
        "3"
    );
}

#[test]
fn immutable_capture_copy_shares_the_existing_environment() {
    let mut session = DebugSession::new(assignment_executable()).expect("debug session");
    stop_with_functions(&mut session);
    let frame = session.stack(0, 1).expect("stack").items[0].id;
    let source = runtime(&session, &name("Captured"), frame);
    session
        .set_expression(&root("Current"), &name("Captured"), Some(frame))
        .expect("copy captured closure");
    let frame = session.stack(0, 1).expect("fresh stack").items[0].id;
    let copied = runtime(&session, &name("Current"), frame);
    assert!(
        function_identity(&source, &copied),
        "assignment must clone SharedFunction, not rebuild captures"
    );
    assert_eq!(
        session
            .evaluate(&call("Current", 1), Some(frame))
            .expect("invoke")
            .value,
        "11"
    );
}

#[test]
fn mutable_capture_cell_is_materialized_when_used_as_the_source() {
    let mut session = DebugSession::new(assignment_executable()).expect("debug session");
    stop_with_functions(&mut session);
    let frame = session.stack(0, 1).expect("stack").items[0].id;
    session
        .set_expression(&root("Current"), &name("CellSlot"), Some(frame))
        .expect("copy mutable capture cell");
    let frame = session.stack(0, 1).expect("fresh stack").items[0].id;
    assert_eq!(
        session
            .evaluate(&call("Current", 1), Some(frame))
            .expect("invoke copied capture")
            .value,
        "2"
    );
}

#[test]
fn uninitialized_local_and_global_roots_accept_one_function_value() {
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
            .set_variable(locals, "Current", &name("Backup"))
            .expect("init local")
            .value,
        "<function add_two>"
    );
    let frame = session.stack(0, 1).expect("after local").items[0].id;
    session
        .set_expression(&root("G"), &name("Backup"), Some(frame))
        .expect("init global");
    let globals = scope_reference(&mut session, "Globals");
    assert_eq!(
        named(
            &session.variables(globals, 0, 10).expect("globals").items,
            "G"
        )
        .value,
        "<function add_two>"
    );
    let frame = session.stack(0, 1).expect("after global").items[0].id;
    assert_eq!(
        session
            .evaluate(&name("Current"), Some(frame))
            .expect("initialized")
            .value,
        "<function add_two>"
    );
}

#[test]
fn parameter_capture_global_and_descendant_destinations_keep_ownership_rules() {
    let mut session = DebugSession::new(assignment_executable()).expect("debug session");
    stop_with_functions(&mut session);
    let frame = session.stack(0, 1).expect("stack").items[0].id;
    session
        .set_expression(&root("Current"), &name("Backup"), Some(frame))
        .expect("local");
    let frame = session.stack(0, 1).expect("stack").items[0].id;
    session
        .set_expression(&field("Box", "Callback"), &name("Backup"), Some(frame))
        .expect("record field");
    let frame = session.stack(0, 1).expect("stack").items[0].id;
    session
        .set_expression(
            &DebugAssignmentTarget {
                root: "Items".to_string(),
                selectors: vec![DebugAssignmentSelector::Index(DebugExpression::Integer(0))],
            },
            &name("Backup"),
            Some(frame),
        )
        .expect("array element");
    let frame = session.stack(0, 1).expect("stack").items[0].id;
    session
        .set_expression(
            &DebugAssignmentTarget {
                root: "Scores".to_string(),
                selectors: vec![DebugAssignmentSelector::Index(DebugExpression::String(
                    "a".to_string(),
                ))],
            },
            &name("Backup"),
            Some(frame),
        )
        .expect("dictionary value");
    let frame = session.stack(0, 1).expect("stack").items[0].id;
    session
        .set_expression(&field("Optional", "value"), &name("Backup"), Some(frame))
        .expect("active payload");
    let captures = scope_reference(&mut session, "Captures");
    session
        .set_variable(captures, "CellSlot", &name("Backup"))
        .expect("capture cell");
    stopped(session.step_into().expect("enter helper"));
    let parameters = scope_reference(&mut session, "Parameters");
    session
        .set_variable(parameters, "Arg", &name("G"))
        .expect("mutable parameter");
    let frame = session.stack(0, 1).expect("helper").items[0].id;
    assert_eq!(
        session
            .evaluate(&call("Arg", 4), Some(frame))
            .expect("param invoke")
            .value,
        "6"
    );
}

#[test]
fn source_lookup_follows_lexical_shadowing_and_globals_only_frames() {
    let mut session = DebugSession::new(assignment_executable()).expect("debug session");
    stop_with_functions(&mut session);
    let frame = session.stack(0, 1).expect("stack").items[0].id;
    session
        .set_expression(&root("Current"), &name("Shared"), Some(frame))
        .expect("local Shared shadows global");
    let frame = session.stack(0, 1).expect("fresh").items[0].id;
    assert_eq!(
        session
            .evaluate(&call("Current", 1), Some(frame))
            .expect("local source")
            .value,
        "3"
    );
    let missing = session
        .set_expression(&root("G"), &name("Captured"), None)
        .expect_err("globals-only source lookup");
    assert_eq!(missing.kind, DebugErrorKind::UnknownName);
    session
        .set_expression(&root("G"), &name("Backup"), Some(frame))
        .expect("frame selects local Backup for a global destination");
}

#[test]
fn signature_and_source_shape_failures_are_actionable() {
    let mut session = DebugSession::new(assignment_executable()).expect("debug session");
    stop_with_functions(&mut session);
    let frame = session.stack(0, 1).expect("stack").items[0].id;
    let locals = scope_reference(&mut session, "Locals");
    let generation = locals;
    let mismatch = session
        .set_expression(&root("Current"), &name("Wrong"), Some(frame))
        .expect_err("signature");
    assert_eq!(mismatch.kind, DebugErrorKind::VariableValueType);
    assert!(mismatch.hint.contains("signature"), "{}", mismatch.hint);
    let missing = session
        .set_expression(&root("Current"), &name("MissingName"), Some(frame))
        .expect_err("unknown");
    assert_eq!(missing.kind, DebugErrorKind::UnknownName);
    let mut empty = DebugSession::new(assignment_executable()).expect("empty source");
    stop_before_current(&mut empty);
    let empty_frame = empty.stack(0, 1).expect("stack").items[0].id;
    let uninit = empty
        .set_expression(&root("Wrong"), &name("Current"), Some(empty_frame))
        .expect_err("uninitialized source");
    assert_eq!(uninit.kind, DebugErrorKind::UninitializedValue);
    let dynamic = session
        .set_expression(&root("Current"), &name("Loose"), Some(frame))
        .expect_err("dynamic source");
    assert_eq!(dynamic.kind, DebugErrorKind::VariableValueType);
    assert!(dynamic.hint.contains("Dynamic"), "{}", dynamic.hint);
    let integer = session
        .set_expression(&root("Current"), &name("Number"), Some(frame))
        .expect_err("non-function");
    assert_eq!(integer.kind, DebugErrorKind::VariableValueType);
    let call = session
        .set_expression(&root("Current"), &call("Backup", 1), Some(frame))
        .expect_err("call");
    assert_eq!(call.kind, DebugErrorKind::VariableValueType);
    assert!(call.hint.contains("AddTwo"), "{}", call.hint);
    let field = session
        .set_expression(
            &root("Current"),
            &DebugExpression::Field {
                base: Box::new(name("Box")),
                name: "Callback".to_string(),
            },
            Some(frame),
        )
        .expect_err("unknown qualified routine");
    assert_eq!(field.kind, DebugErrorKind::UnknownName);
    assert_eq!(
        named(
            &session
                .variables(generation, 0, 30)
                .expect("preserved")
                .items,
            "Current"
        )
        .value,
        "<function add_one>"
    );
}

#[test]
fn same_task_bound_copy_is_allowed_while_nested_forbidden_captures_are_rejected() {
    let mut session = DebugSession::new(assignment_executable()).expect("debug session");
    stop_with_functions(&mut session);
    let frame = session.stack(0, 1).expect("stack").items[0].id;
    let bound = session
        .set_expression(&root("Current"), &name("Bound"), Some(frame))
        .expect("same-task bound copy");
    assert_eq!(bound.value, "<function adder>");
    let frame = session.stack(0, 1).expect("fresh stack").items[0].id;
    let nested = session
        .set_expression(&root("Current"), &name("NestedCell"), Some(frame))
        .expect_err("nested cell");
    assert_eq!(nested.kind, DebugErrorKind::VariableValueType);
    assert!(nested.message.contains("cell"), "{nested:?}");
    session
        .set_expression(&root("Bound"), &name("Backup"), Some(frame))
        .expect("task-bound destination still accepts a safe source");
}

#[test]
fn inactive_variant_function_payload_and_dynamic_destination_remain_rejected() {
    let mut session = DebugSession::new(assignment_executable()).expect("debug session");
    stop_with_functions(&mut session);
    let frame = session.stack(0, 1).expect("stack").items[0].id;
    let transition = session
        .set_expression(
            &DebugAssignmentTarget {
                root: "Missing".to_string(),
                selectors: vec![
                    DebugAssignmentSelector::Field("Some".to_string()),
                    DebugAssignmentSelector::Field("value".to_string()),
                ],
            },
            &name("Backup"),
            Some(frame),
        )
        .expect_err("inactive payload");
    assert_eq!(transition.kind, DebugErrorKind::VariableValueType);
    assert!(
        transition.message.contains("inactive variant")
            || transition.hint.contains("existing function-typed"),
        "{transition:?}"
    );
    let dynamic = session
        .set_expression(&root("Loose"), &name("Backup"), Some(frame))
        .expect_err("dynamic dest");
    assert_eq!(dynamic.kind, DebugErrorKind::VariableValueType);
}

#[test]
fn immutable_hidden_stale_and_unsupported_targets_keep_existing_errors() {
    let mut session = DebugSession::new(assignment_executable()).expect("debug session");
    stop_with_functions(&mut session);
    let frame = session.stack(0, 1).expect("stack").items[0].id;
    let locals = scope_reference(&mut session, "Locals");
    assert_eq!(
        session
            .set_expression(&root("Frozen"), &name("Backup"), Some(frame))
            .expect_err("immutable")
            .kind,
        DebugErrorKind::VariableNotMutable
    );
    assert_eq!(
        session
            .set_expression(&root("Hidden"), &name("Backup"), Some(frame))
            .expect_err("hidden")
            .kind,
        DebugErrorKind::VariableTargetUnknown
    );
    assert_eq!(
        session
            .set_expression(&root("Number"), &name("Backup"), Some(frame))
            .expect_err("integer dest")
            .kind,
        DebugErrorKind::VariableValueType
    );
    session
        .set_variable(locals, "Current", &name("Backup"))
        .expect("success");
    assert_eq!(
        session
            .set_variable(locals, "Current", &name("Backup"))
            .expect_err("stale handle")
            .kind,
        DebugErrorKind::VariableTargetExpired
    );
    let stale = frame;
    let frame = session.stack(0, 1).expect("fresh").items[0].id;
    assert_eq!(
        session
            .set_expression(&root("Current"), &name("Backup"), Some(stale))
            .expect_err("stale frame")
            .kind,
        DebugErrorKind::UnknownFrame
    );
    assert_eq!(
        session
            .evaluate(&call("Current", 1), Some(frame))
            .expect("continuation")
            .value,
        "3"
    );
}

#[test]
fn capture_graph_limits_apply_before_commit() {
    let mut session = DebugSession::new(assignment_executable()).expect("debug session");
    stop_with_functions(&mut session);
    let frame = session.stack(0, 1).expect("stack").items[0].id;
    let error = session
        .set_expression_with_limits(
            &root("Current"),
            &name("Captured"),
            Some(frame),
            DebugEvaluationLimits {
                max_depth: 0,
                ..DebugEvaluationLimits::default()
            },
        )
        .expect_err("depth");
    assert_eq!(error.kind, DebugErrorKind::EvaluationLimit);
    let frame = session.stack(0, 1).expect("unchanged").items[0].id;
    assert_eq!(
        session
            .evaluate(&call("Current", 1), Some(frame))
            .expect("old value")
            .value,
        "2"
    );
}

#[test]
fn indexed_function_assignment_shares_selector_and_source_operation_budget() {
    let mut session = DebugSession::new(assignment_executable()).expect("debug session");
    stop_with_functions(&mut session);
    let target = DebugAssignmentTarget {
        root: "Items".to_string(),
        selectors: vec![DebugAssignmentSelector::Index(DebugExpression::Integer(0))],
    };
    let frame = session.stack(0, 1).expect("stack").items[0].id;
    let exhausted = session
        .set_expression_with_limits(
            &target,
            &name("Backup"),
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
            &name("Backup"),
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
            .expect("invoke replaced element")
            .value,
        "3"
    );
}
