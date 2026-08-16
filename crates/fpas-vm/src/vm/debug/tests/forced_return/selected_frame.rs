//! Older-frame success, convention, continuation, and register-window cases.

use super::atomicity::fingerprint;
use super::chains::{
    four_level_executable, mixed_function_over_procedure_executable,
    mixed_procedure_over_function_executable, three_level_executable,
};
use super::fixtures::*;

#[test]
fn selected_older_function_unwinds_younger_frames_into_its_caller() {
    let mut session = DebugSession::new(three_level_executable()).expect("debug session");
    let _leaf = stop_in_callee(&mut session, "leaf");
    let branch = frame_at_depth(&mut session, 1);
    let before_count = session.test_instruction_count();
    let result = session
        .force_return(branch, Some(&int_expr(99)))
        .expect("selected return");
    assert_eq!(result.value, "99");
    assert_eq!(result.unwound_frames, 2);
    assert_eq!(result.frame.as_ref().expect("caller").name, "root");
    assert_eq!(result.frame.as_ref().expect("caller").depth, 0);
    assert_eq!(session.last_stop().call_depth, 0);
    assert_eq!(session.last_stop().reason, DebugStopReason::Pause);
    assert_eq!(session.test_instruction_count(), before_count);
    let caller = session.stack(0, 1).expect("caller").items[0].id;
    assert_eq!(
        session
            .evaluate(&name("Answer"), Some(caller))
            .expect("selected destination")
            .value,
        "99"
    );
}

#[test]
fn unwind_count_is_selected_depth_plus_one() {
    for depth in 0..=2 {
        let mut session = DebugSession::new(four_level_executable()).expect("debug session");
        let _gamma = stop_in_callee(&mut session, "gamma");
        let selected = frame_at_depth(&mut session, depth);
        let result = session
            .force_return(selected, Some(&int_expr(1)))
            .expect("unwind");
        assert_eq!(result.unwound_frames, depth + 1, "depth {depth}");
        assert_eq!(
            session.last_stop().call_depth,
            3 - (depth + 1),
            "depth {depth}"
        );
        assert_eq!(session.stack(0, 16).expect("stack").total, 4 - (depth + 1));
        assert_eq!(result.frame.as_ref().expect("caller").depth, 0);
    }
}

#[test]
fn selected_expression_reads_selected_bindings_not_younger_ones() {
    let mut session = DebugSession::new(three_level_executable()).expect("debug session");
    let _leaf = stop_in_callee(&mut session, "leaf");
    let branch = frame_at_depth(&mut session, 1);
    let result = session
        .force_return(branch, Some(&name("Local")))
        .expect("selected local");
    assert_eq!(result.value, "11");
    let mut younger = DebugSession::new(three_level_executable()).expect("debug session");
    let _leaf = stop_in_callee(&mut younger, "leaf");
    let branch = frame_at_depth(&mut younger, 1);
    assert_eq!(
        younger
            .force_return(branch, Some(&name("Inner")))
            .expect_err("younger binding")
            .kind,
        DebugErrorKind::UnknownName
    );
}

#[test]
fn selected_procedure_convention_ignores_the_active_function() {
    let mut session =
        DebugSession::new(mixed_procedure_over_function_executable()).expect("debug session");
    let _inner = stop_in_callee(&mut session, "inner");
    let middle = frame_at_depth(&mut session, 1);
    assert_eq!(
        session
            .force_return(middle, Some(&int_expr(1)))
            .expect_err("unexpected")
            .kind,
        DebugErrorKind::FrameReturnValueUnexpected
    );
    let result = session
        .force_return(middle, None)
        .expect("procedure selected");
    assert_eq!(result.value, "()");
    assert_eq!(result.unwound_frames, 2);
    assert_eq!(result.frame.as_ref().expect("caller").name, "root");
}

#[test]
fn selected_function_convention_ignores_the_active_procedure() {
    let mut session =
        DebugSession::new(mixed_function_over_procedure_executable()).expect("debug session");
    let _inner = stop_in_callee(&mut session, "inner");
    let middle = frame_at_depth(&mut session, 1);
    assert_eq!(
        session
            .force_return(middle, None)
            .expect_err("required")
            .kind,
        DebugErrorKind::FrameReturnValueRequired
    );
    let result = session
        .force_return(middle, Some(&int_expr(99)))
        .expect("function selected");
    assert_eq!(result.value, "99");
    assert_eq!(result.unwound_frames, 2);
    let caller = session.stack(0, 1).expect("caller").items[0].id;
    assert_eq!(
        session
            .evaluate(&name("Answer"), Some(caller))
            .expect("destination")
            .value,
        "99"
    );
}

#[test]
fn only_the_selected_caller_destination_is_written() {
    let mut session = DebugSession::new(three_level_executable()).expect("debug session");
    let _leaf = stop_in_callee(&mut session, "leaf");
    let branch = frame_at_depth(&mut session, 1);
    let task_id = session.last_stop().task_id;
    session
        .force_return(branch, Some(&int_expr(99)))
        .expect("selected return");
    let registers = session
        .test_worker_registers(task_id)
        .expect("caller registers");
    assert_eq!(registers.0, 0);
    assert_eq!(registers.2, 0);
    assert_eq!(registers.3, 0);
    assert_eq!(registers.4.len(), 4);
    assert_eq!(registers.4[0], fpas_bytecode::Value::Integer(99));
    assert_eq!(registers.4[1], fpas_bytecode::Value::Integer(7));
    assert!(registers.5[0]);
}

#[test]
fn selected_success_clears_younger_windows_and_dispatches_no_work() {
    let mut session = DebugSession::new(three_level_executable()).expect("debug session");
    let _leaf = stop_in_callee(&mut session, "leaf");
    let branch = frame_at_depth(&mut session, 1);
    let task_id = session.last_stop().task_id;
    let before_count = session.test_instruction_count();
    let before_output = session.output().lines.clone();
    let before_tasks = session
        .tasks(0, 8)
        .expect("tasks")
        .items
        .into_iter()
        .map(|task| (task.id, task.state))
        .collect::<Vec<_>>();
    session
        .force_return(branch, Some(&int_expr(5)))
        .expect("selected return");
    assert_eq!(session.test_instruction_count(), before_count);
    assert_eq!(session.output().lines, before_output);
    assert_eq!(
        session
            .tasks(0, 8)
            .expect("tasks")
            .items
            .into_iter()
            .map(|task| (task.id, task.state))
            .collect::<Vec<_>>(),
        before_tasks
    );
    assert_eq!(session.state(), DebugSessionState::Stopped);
    let registers = session
        .test_worker_registers(task_id)
        .expect("caller registers");
    assert_eq!(registers.3, 0);
    assert_eq!(registers.4.len(), 4);
}

#[test]
fn compiled_selected_frame_continues_from_the_caller() {
    let mut session = DebugSession::new(compiled_fixture()).expect("debug session");
    let _leaf = stop_in_callee(&mut session, "leaf");
    let branch = frame_at_depth(&mut session, 1);
    let result = session
        .force_return(branch, Some(&name("Local")))
        .expect("compiled selected return");
    assert_eq!(result.value, "11");
    assert_eq!(result.unwound_frames, 2);
    match session.continue_execution().expect("continue") {
        DebugRunResult::Terminated(_) => {}
        other => panic!("expected termination after selected return, got {other:?}"),
    }
    let output = session.output().lines.join("\n");
    assert!(
        !output.contains("leaf") && !output.contains("branch"),
        "skipped bodies must not run: {output:?}"
    );
    assert!(
        output.contains("11"),
        "caller must observe the result: {output:?}"
    );
}

#[test]
fn selected_capture_and_global_are_visible() {
    const SOURCE: &str = r#"
program SelectedCapture;

var Shared: integer := 100;

function Outer(Value: integer): integer;
  function Mid(): integer;
    function Inner(): integer;
    begin
      var Hidden: integer := 999;
      return Hidden
    end;
  begin
    var MidLocal: integer := Value + 1;
    return Inner()
  end;
begin
  return Mid()
end;

begin
  var Nested: integer := Outer(1)
end.
"#;
    let (program, diagnostics) = fpas_parser::parse(SOURCE);
    assert!(diagnostics.is_empty(), "{diagnostics:?}");
    let executable = fpas_compiler::compile(&program).expect("compile capture fixture");
    let mut session = DebugSession::new(executable).expect("debug session");
    for _ in 0..64 {
        if session.last_stop().call_depth >= 3 {
            break;
        }
        let _ = stopped(session.step_into().expect("step into nested"));
    }
    assert!(
        session.last_stop().call_depth >= 3,
        "expected Inner to be active"
    );
    let mid = frame_at_depth(&mut session, 1);
    assert_eq!(
        session
            .force_return(mid, Some(&name("Hidden")))
            .expect_err("younger")
            .kind,
        DebugErrorKind::UnknownName
    );
    let result = session
        .force_return(
            mid,
            Some(&DebugExpression::Binary {
                operation: DebugBinaryOperation::Add,
                left: Box::new(DebugExpression::Binary {
                    operation: DebugBinaryOperation::Add,
                    left: Box::new(name("Value")),
                    right: Box::new(name("MidLocal")),
                }),
                right: Box::new(name("Shared")),
            }),
        )
        .expect("capture and global");
    assert_eq!(result.value, "103");
}

#[test]
fn returning_an_older_frame_into_the_entry_caller_is_allowed() {
    let mut session = DebugSession::new(function_return_executable()).expect("debug session");
    let _callee = stop_in_callee(&mut session, "compute");
    let compute = frame_at_depth(&mut session, 0);
    let result = session
        .force_return(compute, Some(&int_expr(4)))
        .expect("into entry");
    assert_eq!(result.frame.as_ref().expect("caller").name, "root");
    assert_eq!(result.unwound_frames, 1);
    assert_eq!(session.last_stop().call_depth, 0);
}

#[test]
fn selected_type_mismatch_is_atomic() {
    let mut session = DebugSession::new(three_level_executable()).expect("debug session");
    let _leaf = stop_in_callee(&mut session, "leaf");
    let branch = frame_at_depth(&mut session, 1);
    let task_id = session.last_stop().task_id;
    let before = fingerprint(&mut session, task_id);
    assert_eq!(
        session
            .force_return(branch, Some(&DebugExpression::String("nope".into())))
            .expect_err("type")
            .kind,
        DebugErrorKind::FrameReturnType
    );
    assert_eq!(fingerprint(&mut session, task_id), before);
}
