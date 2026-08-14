//! Eligibility, path, type, safety, and rollback coverage.

use super::*;

#[test]
fn eligibility_rejects_initialized_immutable_parameter_capture_and_root_only_targets() {
    let mut session = DebugSession::new(assignment_executable()).expect("debug session");
    let frame = session.stack(0, 1).expect("stack").items[0].id;
    assert_eq!(
        session
            .initialize_storage(
                &root("Count"),
                &DebugExpression::Integer(1),
                &DebugExpression::Integer(2),
                Some(frame),
            )
            .expect_err("root-only")
            .kind,
        DebugErrorKind::VariablePathUnsupported
    );
    assert_eq!(
        session
            .initialize_storage(
                &field("Count", "Nope"),
                &DebugExpression::Integer(1),
                &DebugExpression::Integer(2),
                Some(frame),
            )
            .expect_err("integer has no field")
            .kind,
        DebugErrorKind::VariablePathUnsupported
    );
    session
        .set_expression(&root("Count"), &DebugExpression::Integer(8), Some(frame))
        .expect("ordinary complete-root assignment remains available");
    let frame = session.stack(0, 1).expect("after root").items[0].id;
    session
        .set_expression(
            &root("Nested"),
            &DebugExpression::Record(vec![
                ("X".to_string(), DebugExpression::Integer(1)),
                ("Y".to_string(), DebugExpression::Integer(2)),
            ]),
            Some(frame),
        )
        .expect("complete nested root");
    let frame = session.stack(0, 1).expect("after nested").items[0].id;
    assert_eq!(
        session
            .initialize_storage(
                &field("Nested", "X"),
                &DebugExpression::Record(vec![
                    ("X".to_string(), DebugExpression::Integer(1)),
                    ("Y".to_string(), DebugExpression::Integer(2)),
                ]),
                &DebugExpression::Integer(3),
                Some(frame),
            )
            .expect_err("initialized")
            .kind,
        DebugErrorKind::StorageAlreadyInitialized
    );

    let mut empty = DebugSession::new(assignment_executable()).expect("empty session");
    let frame = empty.stack(0, 1).expect("stack").items[0].id;
    assert_eq!(
        empty
            .initialize_storage(
                &field("Frozen", "X"),
                &DebugExpression::Integer(1),
                &DebugExpression::Integer(2),
                Some(frame),
            )
            .expect_err("immutable")
            .kind,
        DebugErrorKind::VariableNotMutable
    );
    assert_eq!(
        empty
            .initialize_storage(
                &field("Arg", "X"),
                &DebugExpression::Integer(1),
                &DebugExpression::Integer(2),
                Some(frame),
            )
            .expect_err("parameter")
            .kind,
        DebugErrorKind::VariableUnavailable
    );
    assert_eq!(
        empty
            .initialize_storage(
                &field("Captured", "X"),
                &DebugExpression::Integer(1),
                &DebugExpression::Integer(2),
                Some(frame),
            )
            .expect_err("capture")
            .kind,
        DebugErrorKind::VariableUnavailable
    );
}

#[test]
fn expired_running_and_foreign_frames_are_rejected() {
    let mut session = DebugSession::new(construction_executable()).expect("debug session");
    let frame = stop_with_empty(&mut session, "State");
    session
        .initialize_storage(
            &nested("State", &["Count"]),
            &make_initial_state(),
            &DebugExpression::Integer(1),
            Some(frame),
        )
        .expect("first commit");
    assert_eq!(
        session
            .initialize_storage(
                &nested("Origin", &["X"]),
                &DebugExpression::Record(vec![
                    ("X".to_string(), DebugExpression::Integer(1)),
                    ("Y".to_string(), DebugExpression::Integer(2)),
                ]),
                &DebugExpression::Integer(3),
                Some(frame),
            )
            .expect_err("expired frame")
            .kind,
        DebugErrorKind::UnknownFrame
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
    let main = tasks.stack_for_task(0, 0, 1).expect("main stack").items[0].id;
    assert_eq!(
        tasks
            .initialize_storage(
                &field("Count", "Nope"),
                &DebugExpression::Integer(1),
                &DebugExpression::Integer(2),
                Some(main),
            )
            .expect_err("main has no Count")
            .kind,
        DebugErrorKind::VariableTargetUnknown
    );

    let mut failed = DebugSession::new(super::panic_executable()).expect("failed");
    let _ = stopped(failed.step_into().expect("step to panic"));
    let _ = stopped(failed.continue_execution().expect("runtime failure"));
    assert_eq!(
        failed
            .initialize_storage(
                &field("Count", "X"),
                &DebugExpression::Integer(1),
                &DebugExpression::Integer(2),
                None,
            )
            .expect_err("failed state")
            .kind,
        DebugErrorKind::InvalidState
    );
}

#[test]
fn missing_paths_type_errors_and_identity_bearing_seeds_preserve_empty_storage() {
    let mut session = DebugSession::new(construction_executable()).expect("debug session");
    let frame = stop_with_empty(&mut session, "State");
    let locals = scope_reference(&mut session, "Locals");
    assert_eq!(
        session
            .initialize_storage(
                &index_target("Items", DebugExpression::Integer(9)),
                &DebugExpression::Array(vec![DebugExpression::Integer(1)]),
                &DebugExpression::Integer(2),
                Some(frame),
            )
            .expect_err("out of range")
            .kind,
        DebugErrorKind::VariableTargetUnknown
    );
    assert_eq!(
        session
            .initialize_storage(
                &index_target("Scores", DebugExpression::String("missing".to_string())),
                &DebugExpression::Dictionary(vec![(
                    DebugExpression::String("red".to_string()),
                    DebugExpression::Integer(1),
                )]),
                &DebugExpression::Integer(2),
                Some(frame),
            )
            .expect_err("missing key")
            .kind,
        DebugErrorKind::VariableTargetUnknown
    );
    assert_eq!(
        session
            .initialize_storage(
                &nested("Selected", &["Value"]),
                &DebugExpression::Call {
                    callee: Box::new(DebugExpression::Callable("Choice.Empty".to_string())),
                    arguments: Vec::new(),
                },
                &DebugExpression::Integer(1),
                Some(frame),
            )
            .expect_err("inactive payload")
            .kind,
        DebugErrorKind::VariablePathUnsupported
    );
    assert_eq!(
        session
            .initialize_storage(
                &nested("State", &["Count"]),
                &DebugExpression::Integer(1),
                &DebugExpression::Integer(2),
                Some(frame),
            )
            .expect_err("seed type")
            .kind,
        DebugErrorKind::VariableValueType
    );
    assert_eq!(
        session
            .initialize_storage_with_limits(
                &nested("State", &["Count"]),
                &make_initial_state(),
                &DebugExpression::Integer(2),
                Some(frame),
                DebugEvaluationLimits {
                    max_operations: 1,
                    ..DebugEvaluationLimits::default()
                },
            )
            .expect_err("shared budget")
            .kind,
        DebugErrorKind::EvaluationLimit
    );
    session.evaluation_cancel_handle().cancel();
    assert_eq!(
        session
            .initialize_storage(
                &nested("State", &["Count"]),
                &make_initial_state(),
                &DebugExpression::Integer(2),
                Some(frame),
            )
            .expect_err("cancelled initializer")
            .kind,
        DebugErrorKind::CallCancelled
    );
    assert_eq!(
        session
            .initialize_storage(
                &nested("State", &["Count"]),
                &DebugExpression::Call {
                    callee: Box::new(DebugExpression::Callable("WriteLn".to_string())),
                    arguments: vec![DebugExpression::Integer(1)],
                },
                &DebugExpression::Integer(2),
                Some(frame),
            )
            .expect_err("forbidden effect")
            .kind,
        DebugErrorKind::ForbiddenCallEffect
    );
    assert_eq!(
        named(
            &session
                .variables(locals, 0, 20)
                .expect("preserved handles")
                .items,
            "State"
        )
        .value,
        "<uninitialized>"
    );
}

#[test]
fn identity_bearing_seeds_and_hidden_names_are_rejected() {
    let mut session = DebugSession::new(construction_executable()).expect("debug session");
    let _ = stop_with_initialized(&mut session, "Callback");
    let frame = session.stack(0, 1).expect("stack").items[0].id;
    assert_eq!(local_value(&mut session, "Boxed"), "<uninitialized>");
    assert_eq!(
        session
            .initialize_storage(
                &nested("Boxed", &["Action"]),
                &DebugExpression::Record(vec![(
                    "Action".to_string(),
                    DebugExpression::Name("Callback".to_string()),
                )]),
                &DebugExpression::Name("Callback".to_string()),
                Some(frame),
            )
            .expect_err("function seed")
            .kind,
        DebugErrorKind::VariableValueType
    );
    assert_eq!(local_value(&mut session, "Boxed"), "<uninitialized>");

    let mut functions =
        DebugSession::new(super::function_assignment_executable()).expect("function fixture");
    super::stop_with_functions(&mut functions);
    let frame = functions.stack(0, 1).expect("stack").items[0].id;
    assert_eq!(
        functions
            .initialize_storage(
                &root("Hidden"),
                &DebugExpression::Integer(1),
                &DebugExpression::Integer(2),
                Some(frame),
            )
            .expect_err("hidden")
            .kind,
        DebugErrorKind::VariableTargetUnknown
    );
}

#[test]
fn response_render_failures_preserve_empty_storage_and_handles() {
    let mut session = DebugSession::new(construction_executable()).expect("debug session");
    let frame = stop_with_empty(&mut session, "State");
    let locals = scope_reference(&mut session, "Locals");
    let error = session
        .initialize_storage_with_limits(
            &nested("State", &["Count"]),
            &make_initial_state(),
            &DebugExpression::Integer(42),
            Some(frame),
            DebugEvaluationLimits {
                max_output_bytes: 1,
                ..DebugEvaluationLimits::default()
            },
        )
        .expect_err("render limit");
    assert_eq!(error.kind, DebugErrorKind::EvaluationLimit);
    assert_eq!(
        named(
            &session
                .variables(locals, 0, 20)
                .expect("preserved handles")
                .items,
            "State"
        )
        .value,
        "<uninitialized>"
    );
}
