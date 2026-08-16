//! Stop-owner, peer-task, entry, and runtime-error cases.

use super::atomicity::fingerprint;
use super::chains::four_level_executable;
use super::fixtures::*;

#[test]
fn stale_and_peer_frames_are_rejected() {
    let mut session = DebugSession::new(function_return_executable()).expect("debug session");
    let callee = stop_in_callee(&mut session, "compute");
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
    let task_id = spawned.last_stop().task_id;
    let before = fingerprint(&mut spawned, task_id);
    assert_eq!(
        spawned
            .force_return(peer, Some(&int_expr(1)))
            .expect_err("peer")
            .kind,
        DebugErrorKind::FrameReturnUnsupported
    );
    assert_eq!(fingerprint(&mut spawned, task_id), before);
    let _ = callee;
}

#[test]
fn root_entry_completion_is_typed_atomic_and_terminal() {
    let mut entry = DebugSession::new(function_return_executable()).expect("debug session");
    let frame = entry.stack(0, 1).expect("entry").items[0].id;
    let task_id = entry.last_stop().task_id;
    let before = fingerprint(&mut entry, task_id);
    let error = entry
        .force_return(frame, Some(&int_expr(1)))
        .expect_err("root procedure rejects a result");
    assert_eq!(error.kind, DebugErrorKind::FrameReturnValueUnexpected);
    assert_eq!(fingerprint(&mut entry, task_id), before);
    let instructions = entry.test_instruction_count();
    let result = entry
        .force_return(frame, None)
        .expect("complete root entry");
    assert!(result.terminated);
    assert!(result.frame.is_none());
    assert_eq!(result.unwound_frames, 1);
    assert_eq!(entry.state(), DebugSessionState::Terminated);
    assert_eq!(entry.test_instruction_count(), instructions);
}

#[test]
fn root_entry_completion_cancels_a_spawned_peer_without_dispatch() {
    let mut session = DebugSession::new(spawn_then_call_executable()).expect("debug session");
    let _callee = stop_in_callee(&mut session, "compute");
    let _ = session.take_task_events();
    let root = frame_at_depth(&mut session, 1);
    let instructions = session.test_instruction_count();
    let result = session.force_return(root, None).expect("complete root");
    assert!(result.terminated);
    assert_eq!(session.test_instruction_count(), instructions);
    assert!(
        session
            .take_task_events()
            .iter()
            .any(|event| { event.task_id == 1 && event.kind == crate::DebugTaskEventKind::Exited })
    );
    assert!(matches!(
        session.test_poll_task_result(1),
        crate::vm::TaskResultPoll::Failed(_)
    ));
}

#[test]
fn child_entry_completion_publishes_exit_and_retargets_the_stop() {
    let mut session = DebugSession::new(spawn_then_call_executable()).expect("debug session");
    let _callee = stop_in_callee(&mut session, "compute");
    let _ = session.take_task_events();
    let _ = stopped(session.step_into_task(1).expect("stop in child"));
    assert_eq!(session.last_stop().task_id, 1);
    let child = session.stack(0, 1).expect("child entry").items[0].id;
    let instructions = session.test_instruction_count();
    let result = session.force_return(child, None).expect("complete child");
    assert!(!result.terminated);
    assert!(result.frame.is_none());
    assert_eq!(session.state(), DebugSessionState::Stopped);
    assert_eq!(session.last_stop().task_id, 0);
    assert_eq!(session.test_instruction_count(), instructions);
    assert!(
        session
            .take_task_events()
            .iter()
            .any(|event| { event.task_id == 1 && event.kind == crate::DebugTaskEventKind::Exited })
    );
    assert!(matches!(
        session.test_poll_task_result(1),
        crate::vm::TaskResultPoll::Available(fpas_bytecode::Value::Unit)
    ));
    assert!(matches!(
        session.test_poll_task_result(1),
        crate::vm::TaskResultPoll::Consumed
    ));
}

#[test]
fn runtime_error_entry_completion_recovers_and_terminates() {
    let mut failed = DebugSession::new(panic_executable()).expect("debug session");
    let _ = stopped(failed.step_into().expect("step to panic"));
    let _ = stopped(failed.continue_execution().expect("runtime failure"));
    assert_eq!(failed.state(), DebugSessionState::Failed);
    let frame = failed.stack(0, 1).expect("failed stack").items[0].id;
    let result = failed
        .force_return(frame, None)
        .expect("recover root entry");
    assert!(result.terminated);
    assert_eq!(failed.state(), DebugSessionState::Terminated);
}

#[test]
fn runtime_error_callee_recovery_is_typed_atomic_and_resumable() {
    let mut session = DebugSession::new(callee_panic_executable()).expect("debug session");
    let _callee = stop_in_callee(&mut session, "compute");
    let _ = stopped(session.continue_execution().expect("runtime failure"));
    assert_eq!(session.state(), DebugSessionState::Failed);
    assert_eq!(session.last_stop().reason, DebugStopReason::RuntimeError);
    let callee = session.stack(0, 1).expect("failed callee").items[0].id;
    let task_id = session.last_stop().task_id;
    let before = fingerprint(&mut session, task_id);
    let error = session
        .force_return(callee, Some(&DebugExpression::String("nope".into())))
        .expect_err("type mismatch");
    assert_eq!(error.kind, DebugErrorKind::FrameReturnType);
    assert_eq!(fingerprint(&mut session, task_id), before);

    let instructions = session.test_instruction_count();
    let result = session
        .force_return(callee, Some(&int_expr(9)))
        .expect("recover failed callee");
    assert!(!result.terminated);
    assert_eq!(session.state(), DebugSessionState::Stopped);
    assert_eq!(session.last_stop().reason, DebugStopReason::Pause);
    assert!(session.last_stop().diagnostic.is_none());
    assert_eq!(session.test_instruction_count(), instructions);
    assert!(matches!(
        session.continue_execution().expect("resume recovered root"),
        DebugRunResult::Terminated(_)
    ));
}

#[test]
fn retained_child_failure_entry_recovery_replaces_the_exact_result() {
    let mut session = DebugSession::new(spawn_failing_task_executable()).expect("debug session");
    let _compute = stop_in_callee(&mut session, "compute");
    let _ = stopped(session.step_into_task(1).expect("child entry"));
    let _ = stopped(session.step_into_task(1).expect("child failure"));
    assert_eq!(session.state(), DebugSessionState::Failed);
    assert_eq!(session.last_stop().task_id, 1);
    assert!(matches!(
        session.test_poll_task_result(1),
        crate::vm::TaskResultPoll::Failed(_)
    ));
    let child = session.stack(0, 1).expect("failed child").items[0].id;
    let instructions = session.test_instruction_count();
    let result = session
        .force_return(child, None)
        .expect("recover child entry");
    assert!(!result.terminated);
    assert_eq!(session.state(), DebugSessionState::Stopped);
    assert_eq!(session.last_stop().task_id, 0);
    assert_eq!(session.test_instruction_count(), instructions);
    assert!(matches!(
        session.test_poll_task_result(1),
        crate::vm::TaskResultPoll::Available(fpas_bytecode::Value::Unit)
    ));
    assert!(matches!(
        session.test_poll_task_result(1),
        crate::vm::TaskResultPoll::Consumed
    ));
}

#[test]
fn four_level_entry_frame_is_rejected_without_state_change() {
    let mut session = DebugSession::new(four_level_executable()).expect("debug session");
    let _gamma = stop_in_callee(&mut session, "gamma");
    let entry = frame_at_depth(&mut session, 3);
    let task_id = session.last_stop().task_id;
    let before = fingerprint(&mut session, task_id);
    let error = session
        .force_return(entry, Some(&int_expr(1)))
        .expect_err("root procedure rejects value");
    assert_eq!(error.kind, DebugErrorKind::FrameReturnValueUnexpected);
    assert_eq!(fingerprint(&mut session, task_id), before);
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
