//! Session-level forced-return coverage.

use super::*;
use crate::DebugStop;

fn fingerprint(
    session: &DebugSession,
    task_id: u64,
) -> (
    u64,
    DebugStop,
    Option<(
        u16,
        usize,
        usize,
        usize,
        Vec<fpas_bytecode::Value>,
        Vec<bool>,
    )>,
    Vec<u64>,
) {
    (
        session.test_instruction_count(),
        session.last_stop().clone(),
        session.test_worker_registers(task_id),
        session
            .stack(0, 8)
            .expect("stack")
            .items
            .into_iter()
            .map(|frame| frame.id)
            .collect(),
    )
}

#[test]
fn forced_return_completes_a_scalar_function_and_restores_the_caller() {
    let mut session = DebugSession::new(function_return_executable()).expect("debug session");
    let callee = stop_in_callee(&mut session, "compute");
    let before_count = session.test_instruction_count();
    let result = session
        .force_return(callee, Some(&int_expr(99)))
        .expect("force return");
    assert_eq!(result.value, "99");
    assert_eq!(result.type_name, "integer");
    assert_eq!(result.frame.name, "root");
    assert_eq!(result.frame.depth, 0);
    assert_eq!(session.last_stop().call_depth, 0);
    assert_eq!(session.last_stop().reason, DebugStopReason::Pause);
    assert_eq!(session.test_instruction_count(), before_count);
    let frame = session.stack(0, 1).expect("caller stack").items[0].id;
    assert_eq!(
        session
            .evaluate(&name("Answer"), Some(frame))
            .expect("caller destination")
            .value,
        "99"
    );
}

#[test]
fn caller_continuation_observes_the_forced_function_result() {
    let mut session = DebugSession::new(function_return_executable()).expect("debug session");
    let callee = stop_in_callee(&mut session, "compute");
    session
        .force_return(callee, Some(&int_expr(99)))
        .expect("force return");
    match session.continue_execution().expect("continue") {
        DebugRunResult::Terminated(termination) => {
            assert_eq!(termination.value, fpas_bytecode::Value::Unit);
        }
        other => panic!("expected termination after forced return, got {other:?}"),
    }
}

#[test]
fn forced_return_completes_a_procedure_without_an_expression() {
    let mut session = DebugSession::new(procedure_return_executable()).expect("debug session");
    let callee = stop_in_callee(&mut session, "announce");
    let result = session
        .force_return(callee, None)
        .expect("procedure return");
    assert_eq!(result.value, "()");
    assert_eq!(result.frame.name, "root");
    assert_eq!(session.last_stop().call_depth, 0);
}

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
fn callee_locals_are_readable_during_the_single_pre_commit_evaluation() {
    let mut session = DebugSession::new(function_return_executable()).expect("debug session");
    let _callee = stop_in_callee(&mut session, "compute");
    for _ in 0..8 {
        let locals = scope_reference(&mut session, "Locals");
        if named(
            &session.variables(locals, 0, 10).expect("locals").items,
            "Offset",
        )
        .value
            != "<uninitialized>"
        {
            break;
        }
        let _ = stopped(session.step_into().expect("initialize Offset"));
    }
    let callee = session.stack(0, 1).expect("stack").items[0].id;
    let result = session
        .force_return(
            callee,
            Some(&DebugExpression::Binary {
                operation: DebugBinaryOperation::Add,
                left: Box::new(name("Offset")),
                right: Box::new(int_expr(40)),
            }),
        )
        .expect("evaluate callee local once");
    assert_eq!(result.value, "41");
}

#[test]
fn array_results_validate_structurally() {
    let mut session = DebugSession::new(array_return_executable()).expect("debug session");
    let callee = stop_in_callee(&mut session, "compute");
    let result = session
        .force_return(
            callee,
            Some(&DebugExpression::Array(vec![int_expr(1), int_expr(2)])),
        )
        .expect("array return");
    assert_eq!(result.indexed_variables, 2);
    let frame = session.stack(0, 1).expect("caller").items[0].id;
    assert_eq!(
        session
            .evaluate(&name("Answer"), Some(frame))
            .expect("array destination")
            .indexed_variables,
        2
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
fn stale_caller_and_peer_frames_are_rejected() {
    let mut session = DebugSession::new(function_return_executable()).expect("debug session");
    let callee = stop_in_callee(&mut session, "compute");
    let caller = session.stack(0, 8).expect("stack").items[1].id;
    assert_eq!(
        session
            .force_return(caller, Some(&int_expr(1)))
            .expect_err("caller depth")
            .kind,
        DebugErrorKind::FrameReturnUnsupported
    );
    session
        .force_return(callee, Some(&int_expr(7)))
        .expect("success");
    assert_eq!(
        session
            .force_return(callee, Some(&int_expr(8)))
            .expect_err("stale")
            .kind,
        DebugErrorKind::UnknownFrame
    );

    let mut spawned = DebugSession::new(spawn_then_call_executable()).expect("debug session");
    let callee = stop_in_callee(&mut spawned, "compute");
    spawned.select_task(1).expect("inspect child");
    let peer = spawned.stack(0, 1).expect("peer stack").items[0].id;
    spawned.select_task(0).expect("restore main");
    assert_eq!(
        spawned
            .force_return(peer, Some(&int_expr(1)))
            .expect_err("peer")
            .kind,
        DebugErrorKind::FrameReturnUnsupported
    );
    let _ = callee;
}

#[test]
fn entry_and_runtime_error_stops_are_unsupported() {
    let mut entry = DebugSession::new(function_return_executable()).expect("debug session");
    let frame = entry.stack(0, 1).expect("entry").items[0].id;
    assert_eq!(
        entry
            .force_return(frame, Some(&int_expr(1)))
            .expect_err("entry")
            .kind,
        DebugErrorKind::FrameReturnUnsupported
    );

    let mut failed = DebugSession::new(panic_executable()).expect("debug session");
    let _ = stopped(failed.step_into().expect("step to panic"));
    let _ = stopped(failed.continue_execution().expect("runtime failure"));
    assert_eq!(failed.state(), DebugSessionState::Failed);
    let frame = failed.stack(0, 1).expect("failed stack").items[0].id;
    assert_eq!(
        failed
            .force_return(frame, Some(&int_expr(1)))
            .expect_err("runtime error")
            .kind,
        DebugErrorKind::FrameReturnUnsupported
    );
}

#[test]
fn failure_preserves_frames_registers_and_handles() {
    let mut session = DebugSession::new(function_return_executable()).expect("debug session");
    let callee = stop_in_callee(&mut session, "compute");
    let task_id = session.last_stop().task_id;
    let locals = scope_reference(&mut session, "Locals");
    let before = fingerprint(&session, task_id);
    let before_local = session
        .variables(locals, 0, 10)
        .expect("locals")
        .items
        .clone();
    session
        .force_return(callee, Some(&DebugExpression::String("nope".into())))
        .expect_err("type");
    assert_eq!(fingerprint(&session, task_id), before);
    assert_eq!(
        session.variables(locals, 0, 10).expect("same locals").items,
        before_local
    );
}

#[test]
fn result_rendering_limit_failure_precedes_the_frame_commit() {
    let mut session = DebugSession::new(function_return_executable()).expect("debug session");
    let callee = stop_in_callee(&mut session, "compute");
    let task_id = session.last_stop().task_id;
    let before = fingerprint(&session, task_id);
    let mut limits = DebugEvaluationLimits::default();
    limits.max_output_bytes = 0;

    let error = session
        .force_return_with_limits(callee, Some(&int_expr(99)), limits)
        .expect_err("rendering limit");

    assert_eq!(error.kind, DebugErrorKind::EvaluationLimit);
    assert_eq!(fingerprint(&session, task_id), before);
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
    let before = fingerprint(&session, task_id);

    let error = session
        .force_return(
            callee,
            Some(&DebugExpression::Array(vec![int_expr(1), int_expr(2)])),
        )
        .expect_err("handle limit");

    assert_eq!(error.kind, DebugErrorKind::InspectionLimit);
    assert_eq!(fingerprint(&session, task_id), before);
}

#[test]
fn aggregate_result_uses_the_handle_reserved_during_caller_refresh() {
    let mut inspection_limits = DebugInspectionLimits::default();
    inspection_limits.max_handles = 1;
    let mut session = DebugSession::with_limits(
        array_return_executable(),
        Vec::new(),
        inspection_limits,
        DebugExecutionLimits::default(),
    )
    .expect("debug session");
    let callee = stop_in_callee(&mut session, "compute");

    let result = session
        .force_return(
            callee,
            Some(&DebugExpression::Array(vec![int_expr(1), int_expr(2)])),
        )
        .expect("reserved result handle");

    assert_ne!(result.variables_reference, 0);
    assert_eq!(
        session
            .variables(result.variables_reference, 0, 2)
            .expect("result children")
            .items
            .len(),
        2
    );
}

#[test]
fn success_removes_exactly_one_frame_and_stays_stopped() {
    let mut session = DebugSession::new(function_return_executable()).expect("debug session");
    let callee = stop_in_callee(&mut session, "compute");
    let before_depth = session.last_stop().call_depth;
    let before_count = session.test_instruction_count();
    session
        .force_return(callee, Some(&int_expr(3)))
        .expect("force return");
    assert_eq!(
        session.last_stop().call_depth,
        before_depth.saturating_sub(1)
    );
    assert_eq!(session.test_instruction_count(), before_count);
    assert_eq!(session.state(), DebugSessionState::Stopped);
    assert_eq!(session.stack(0, 8).expect("stack").total, 1);
}

#[test]
fn success_refreshes_every_stopped_snapshot_once() {
    let mut session = DebugSession::new(spawn_then_call_executable()).expect("debug session");
    let callee = stop_in_callee(&mut session, "compute");
    session.select_task(1).expect("child");
    let child_frame = session.stack(0, 1).expect("child stack").items[0].id;
    session.select_task(0).expect("main");
    let stale_locals = scope_reference(&mut session, "Locals");
    session
        .force_return(callee, Some(&int_expr(9)))
        .expect("force return");
    assert_eq!(
        session
            .scopes(child_frame)
            .expect_err("expired child frame")
            .kind,
        DebugErrorKind::UnknownFrame
    );
    assert_eq!(
        session
            .variables(stale_locals, 0, 4)
            .expect_err("expired locals")
            .kind,
        DebugErrorKind::UnknownVariablesReference
    );
    session.select_task(1).expect("fresh child");
    assert!(
        !session
            .stack(0, 1)
            .expect("fresh child stack")
            .items
            .is_empty()
    );
}
