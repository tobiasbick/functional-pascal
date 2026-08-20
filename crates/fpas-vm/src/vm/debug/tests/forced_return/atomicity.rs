//! State fingerprints and structural failure atomicity.

use crate::DebugStop;
use crate::vm::debug::types::DebugTaskState;

use super::chains::three_level_executable;
use super::fixtures::*;

type StateFingerprint = (
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
    Vec<String>,
    Vec<(u64, DebugTaskState)>,
);

pub(super) fn fingerprint(session: &mut DebugSession, task_id: u64) -> StateFingerprint {
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
        session.output().lines.clone(),
        session
            .tasks(0, 8)
            .expect("tasks")
            .items
            .into_iter()
            .map(|task| (task.id, task.state))
            .collect(),
    )
}

#[test]
fn failure_preserves_frames_registers_and_handles() {
    let mut session = DebugSession::new(function_return_executable()).expect("debug session");
    let callee = stop_in_callee(&mut session, "compute");
    let task_id = session.last_stop().task_id;
    let locals = scope_reference(&mut session, "Locals");
    let before = fingerprint(&mut session, task_id);
    let before_local = session
        .variables(locals, 0, 10)
        .expect("locals")
        .items
        .clone();
    session
        .force_return(callee, Some(&DebugExpression::String("nope".into())))
        .expect_err("type");
    assert_eq!(fingerprint(&mut session, task_id), before);
    assert_eq!(
        session.variables(locals, 0, 10).expect("same locals").items,
        before_local
    );
}

#[test]
fn selected_frame_failure_preserves_younger_windows() {
    let mut session = DebugSession::new(three_level_executable()).expect("debug session");
    let _leaf = stop_in_callee(&mut session, "leaf");
    let branch = frame_at_depth(&mut session, 1);
    let task_id = session.last_stop().task_id;
    let before = fingerprint(&mut session, task_id);
    session
        .force_return(branch, Some(&DebugExpression::String("nope".into())))
        .expect_err("type");
    assert_eq!(fingerprint(&mut session, task_id), before);
    assert_eq!(session.stack(0, 8).expect("stack").total, 3);
}
