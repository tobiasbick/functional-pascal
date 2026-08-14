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
fn entry_and_runtime_error_stops_are_unsupported() {
    let mut entry = DebugSession::new(function_return_executable()).expect("debug session");
    let frame = entry.stack(0, 1).expect("entry").items[0].id;
    let task_id = entry.last_stop().task_id;
    let before = fingerprint(&mut entry, task_id);
    let error = entry
        .force_return(frame, Some(&int_expr(1)))
        .expect_err("entry");
    assert_eq!(error.kind, DebugErrorKind::FrameReturnUnsupported);
    assert!(error.message.contains("entry frame"), "{error:?}");
    assert_eq!(fingerprint(&mut entry, task_id), before);

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
    assert_eq!(failed.state(), DebugSessionState::Failed);
    assert!(failed.last_stop().diagnostic.is_some());
}

#[test]
fn four_level_entry_frame_is_rejected_without_state_change() {
    let mut session = DebugSession::new(four_level_executable()).expect("debug session");
    let _gamma = stop_in_callee(&mut session, "gamma");
    let entry = frame_at_depth(&mut session, 3);
    let task_id = session.last_stop().task_id;
    let before = fingerprint(&mut session, task_id);
    assert_eq!(
        session
            .force_return(entry, Some(&int_expr(1)))
            .expect_err("entry")
            .kind,
        DebugErrorKind::FrameReturnUnsupported
    );
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
