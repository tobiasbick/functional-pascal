//! Convention, metadata, type, and resource-limit failures.

use super::atomicity::fingerprint;
use super::fixtures::*;

#[test]
fn functions_require_an_expression_and_procedures_reject_one() {
    let mut function = DebugSession::new(function_return_executable()).expect("debug session");
    let callee = stop_in_callee(&mut function, "compute");
    assert_eq!(
        function
            .force_return(callee, None)
            .expect_err("missing")
            .kind,
        DebugErrorKind::FrameReturnValueRequired
    );
    let mut procedure = DebugSession::new(procedure_return_executable()).expect("debug session");
    let callee = stop_in_callee(&mut procedure, "announce");
    assert_eq!(
        procedure
            .force_return(callee, Some(&int_expr(1)))
            .expect_err("unexpected")
            .kind,
        DebugErrorKind::FrameReturnValueUnexpected
    );
}

#[test]
fn type_and_category_failures_are_actionable() {
    let mut integer = DebugSession::new(function_return_executable()).expect("debug session");
    let callee = stop_in_callee(&mut integer, "compute");
    assert_eq!(
        integer
            .force_return(callee, Some(&DebugExpression::String("nope".into())))
            .expect_err("type")
            .kind,
        DebugErrorKind::FrameReturnType
    );

    let mut dynamic = DebugSession::new(dynamic_result_executable()).expect("debug session");
    let callee = stop_in_callee(&mut dynamic, "compute");
    assert_eq!(
        dynamic
            .force_return(callee, Some(&int_expr(1)))
            .expect_err("dynamic")
            .kind,
        DebugErrorKind::FrameReturnUnsupported
    );

    let mut function = DebugSession::new(function_result_executable()).expect("debug session");
    let callee = stop_in_callee(&mut function, "compute");
    assert_eq!(
        function
            .force_return(callee, Some(&int_expr(1)))
            .expect_err("function")
            .kind,
        DebugErrorKind::FrameReturnUnsupported
    );

    let mut task = DebugSession::new(task_result_executable()).expect("debug session");
    let callee = stop_in_callee(&mut task, "compute");
    assert_eq!(
        task.force_return(callee, Some(&int_expr(1)))
            .expect_err("task")
            .kind,
        DebugErrorKind::FrameReturnUnsupported
    );
}

#[test]
fn missing_result_metadata_is_rejected_without_inference() {
    let mut session = DebugSession::new(metadata_less_executable()).expect("debug session");
    let callee = stop_in_callee(&mut session, "compute");
    let error = session
        .force_return(callee, Some(&int_expr(1)))
        .expect_err("metadata");
    assert_eq!(error.kind, DebugErrorKind::FrameReturnUnsupported);
    assert!(error.message.contains("portable result type"), "{error:?}");
}

#[test]
fn result_rendering_limit_failure_precedes_the_frame_commit() {
    let mut session = DebugSession::new(function_return_executable()).expect("debug session");
    let callee = stop_in_callee(&mut session, "compute");
    let task_id = session.last_stop().task_id;
    let before = fingerprint(&mut session, task_id);
    let mut limits = DebugEvaluationLimits::default();
    limits.max_output_bytes = 0;

    let error = session
        .force_return_with_limits(callee, Some(&int_expr(99)), limits)
        .expect_err("rendering limit");

    assert_eq!(error.kind, DebugErrorKind::EvaluationLimit);
    assert_eq!(fingerprint(&mut session, task_id), before);
}

#[test]
fn aggregate_result_handle_limit_failure_precedes_the_frame_commit() {
    let mut inspection_limits = DebugInspectionLimits::default();
    inspection_limits.max_handles = 0;
    let mut session = DebugSession::with_limits(
        array_return_executable(),
        Vec::new(),
        inspection_limits,
        DebugExecutionLimits::default(),
    )
    .expect("debug session");
    let callee = stop_in_callee(&mut session, "compute");
    let task_id = session.last_stop().task_id;
    let before = fingerprint(&mut session, task_id);

    let error = session
        .force_return(
            callee,
            Some(&DebugExpression::Array(vec![int_expr(1), int_expr(2)])),
        )
        .expect_err("handle limit");

    assert_eq!(error.kind, DebugErrorKind::InspectionLimit);
    assert_eq!(fingerprint(&mut session, task_id), before);
}
