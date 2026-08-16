//! Depth-zero forced-return success regressions.

use super::fixtures::*;

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
    assert_eq!(result.unwound_frames, 1);
    assert_eq!(result.frame.as_ref().expect("caller").name, "root");
    assert_eq!(result.frame.as_ref().expect("caller").depth, 0);
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
    assert_eq!(result.unwound_frames, 1);
    assert_eq!(result.frame.as_ref().expect("caller").name, "root");
    assert_eq!(session.last_stop().call_depth, 0);
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
    assert_eq!(result.unwound_frames, 1);
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
    assert_eq!(result.unwound_frames, 1);
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
    assert_eq!(result.unwound_frames, 1);
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
