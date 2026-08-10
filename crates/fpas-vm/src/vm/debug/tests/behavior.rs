use super::*;

use std::time::{Duration, Instant};

#[test]
fn breakpoints_bind_only_to_sequence_points_and_stop_before_execution() {
    let mut session = DebugSession::new(call_executable()).expect("debug session");
    assert_eq!(session.last_stop().reason, DebugStopReason::Entry);
    assert_eq!(
        session.last_stop().location.as_ref().map(|it| it.line),
        Some(1)
    );
    let bound = session
        .set_breakpoint(SourceBreakpoint {
            source: "test.fpas".to_string(),
            line: 10,
            column: None,
        })
        .expect("breakpoint");
    assert!(bound.is_verified());
    let unbound = session
        .set_breakpoint(SourceBreakpoint {
            source: "test.fpas".to_string(),
            line: 9,
            column: None,
        })
        .expect("unverified breakpoint");
    assert!(!unbound.is_verified());

    let stop = stopped(session.continue_execution().expect("continue"));
    assert_eq!(stop.reason, DebugStopReason::Breakpoint);
    assert_eq!(stop.breakpoint_id, Some(bound.id));
    assert_eq!(stop.location.map(|it| it.line), Some(10));
}

#[test]
fn cooperative_pause_waits_for_a_blocking_intrinsic_to_return() {
    let mut session = DebugSession::new(blocking_intrinsic_executable()).expect("debug session");
    session.pause_handle().request_pause();

    let started = Instant::now();
    let stop = stopped(session.continue_execution().expect("cooperative pause"));

    assert!(started.elapsed() >= Duration::from_millis(20));
    assert_eq!(stop.reason, DebugStopReason::Pause);
    assert_eq!(stop.location.map(|location| location.line), Some(2));
}

#[test]
fn step_into_over_and_out_follow_source_call_depth() {
    let mut into = DebugSession::new(call_executable()).expect("debug session");
    let call = stopped(into.step_into().expect("step to call"));
    assert_eq!(call.location.map(|it| it.line), Some(2));
    let callee = stopped(into.step_into().expect("step into callee"));
    assert_eq!(callee.call_depth, 1);
    assert_eq!(callee.location.map(|it| it.line), Some(10));
    let caller = stopped(into.step_out().expect("step out"));
    assert_eq!(caller.call_depth, 0);
    assert_eq!(caller.location.map(|it| it.line), Some(3));

    let mut over = DebugSession::new(call_executable()).expect("debug session");
    stopped(over.step_into().expect("step to call"));
    let caller = stopped(over.step_over().expect("step over call"));
    assert_eq!(caller.call_depth, 0);
    assert_eq!(caller.location.map(|it| it.line), Some(3));
}

#[test]
fn stepping_distinguishes_same_line_columns_loops_and_recursion() {
    let mut same_line = DebugSession::new(same_line_executable()).expect("debug session");
    let stop = stopped(same_line.step_into().expect("same-line step"));
    let location = stop.location.expect("same-line location");
    assert_eq!((location.line, location.column), (1, 20));

    let mut looping = DebugSession::new(loop_executable()).expect("debug session");
    let stop = stopped(looping.step_into().expect("loop backedge step"));
    assert_eq!(stop.instruction, 0);

    let mut recursive = DebugSession::new(recursive_executable()).expect("debug session");
    let stop = stopped(recursive.step_into().expect("recursive step"));
    assert_eq!(stop.call_depth, 1);
    recursive.disconnect();
    assert_eq!(recursive.state(), DebugSessionState::Terminated);
    assert_eq!(
        recursive.step_into().expect_err("terminated session").kind,
        DebugErrorKind::InvalidState
    );
}

#[test]
fn cooperative_pause_and_runtime_failure_leave_stable_states() {
    let mut paused = DebugSession::new(call_executable()).expect("debug session");
    paused.pause_handle().request_pause();
    let stop = stopped(paused.continue_execution().expect("pause"));
    assert_eq!(stop.reason, DebugStopReason::Pause);
    assert_eq!(paused.state(), DebugSessionState::Stopped);

    let mut failed = DebugSession::new(panic_executable()).expect("debug session");
    stopped(failed.step_into().expect("step to panic"));
    let stop = stopped(failed.continue_execution().expect("runtime failure"));
    assert_eq!(stop.reason, DebugStopReason::RuntimeError);
    assert!(stop.diagnostic.is_some());
    assert_eq!(failed.state(), DebugSessionState::Failed);
    assert_eq!(
        failed
            .continue_execution()
            .expect_err("failed session")
            .kind,
        DebugErrorKind::InvalidState
    );

    let mut before_source = DebugSession::new(panic_without_sequence_point_executable())
        .expect("debug session without points");
    let stop = stopped(
        before_source
            .continue_execution()
            .expect("runtime failure before source point"),
    );
    assert_eq!(stop.reason, DebugStopReason::RuntimeError);
    assert!(stop.location.is_none());
    assert!(stop.diagnostic.is_some());
}

#[test]
fn task_spawning_is_rejected_before_debug_execution() {
    let error = DebugSession::new(task_executable())
        .err()
        .expect("task rejection");
    assert_eq!(error.kind, DebugErrorKind::UnsupportedTasks);
    assert!(error.message.contains("root"));

    let mut allowed =
        DebugSession::new(unreachable_task_executable()).expect("unreachable task is allowed");
    assert!(matches!(
        allowed.continue_execution().expect("root termination"),
        DebugRunResult::Terminated(_)
    ));
}

#[test]
fn stack_scopes_variables_and_expired_references_are_snapshot_stable() {
    let mut session = DebugSession::new(inspection_executable()).expect("debug session");
    stopped(session.step_into().expect("initialize locals"));
    let stack = session.stack(0, 10).expect("stack");
    assert_eq!(stack.total, 1);
    let root_frame = stack.items[0].id;
    let scopes = session.scopes(root_frame).expect("root scopes");
    let locals = scopes
        .iter()
        .find(|scope| scope.name == "Locals")
        .expect("locals scope")
        .variables_reference;
    let globals = scopes
        .iter()
        .find(|scope| scope.name == "Globals")
        .expect("globals scope")
        .variables_reference;
    let first = session.variables(locals, 0, 10).expect("locals");
    let second = session.variables(locals, 0, 10).expect("repeat locals");
    assert_eq!(first, second);
    assert_eq!(
        first
            .items
            .iter()
            .map(|variable| (variable.name.as_str(), variable.value.as_str()))
            .collect::<Vec<_>>(),
        [
            ("Answer", "42"),
            ("Inner", "'boom'"),
            ("Answer", "'boom'"),
            ("Items", "[2 items]")
        ]
    );
    let items_reference = first
        .items
        .iter()
        .find(|variable| variable.name == "Items")
        .expect("array local")
        .variables_reference;
    let array_items = session
        .variables(items_reference, 0, 10)
        .expect("array children");
    assert_eq!(
        array_items
            .items
            .iter()
            .map(|variable| variable.value.as_str())
            .collect::<Vec<_>>(),
        ["1", "2"]
    );
    assert_eq!(
        session.variables(globals, 0, 10).expect("globals").items[0].value,
        "42"
    );

    stopped(session.step_into().expect("enter helper"));
    assert_eq!(
        session.scopes(root_frame).expect_err("expired frame").kind,
        DebugErrorKind::UnknownFrame
    );
    assert_eq!(
        session
            .variables(locals, 0, 10)
            .expect_err("expired variables")
            .kind,
        DebugErrorKind::UnknownVariablesReference
    );
    assert_eq!(
        session
            .variables(items_reference, 0, 10)
            .expect_err("expired child variables")
            .kind,
        DebugErrorKind::UnknownVariablesReference
    );
    let stack = session.stack(0, 10).expect("callee stack");
    assert_eq!(stack.total, 2);
    assert_eq!(
        (stack.items[0].name.as_str(), stack.items[1].name.as_str()),
        ("helper", "root")
    );
    let parameters =
        session.scopes(stack.items[0].id).expect("helper scopes")[0].variables_reference;
    assert_eq!(
        session
            .variables(parameters, 0, 10)
            .expect("parameter")
            .items[0]
            .value,
        "42"
    );
}

#[test]
fn inspection_limits_bound_pages_handles_and_output() {
    let limits = DebugInspectionLimits {
        max_frames: 3,
        ..DebugInspectionLimits::default()
    };
    let mut deep = DebugSession::with_args_and_limits(recursive_executable(), Vec::new(), limits)
        .expect("deep stack session");
    for _ in 0..5 {
        stopped(deep.step_into().expect("recursive step"));
    }
    let stack = deep.stack(0, 3).expect("bounded deep stack");
    assert_eq!(stack.items.len(), 3);
    assert_eq!(stack.total, 6);

    let limits = DebugInspectionLimits {
        max_children: 1,
        ..DebugInspectionLimits::default()
    };
    let mut pages = DebugSession::with_args_and_limits(inspection_executable(), Vec::new(), limits)
        .expect("bounded session");
    stopped(pages.step_into().expect("initialize locals"));
    assert_eq!(
        pages.stack(0, 257).expect_err("frame page limit").kind,
        DebugErrorKind::InspectionLimit
    );
    let frame = pages.stack(0, 1).expect("frame").items[0].id;
    let locals = pages
        .scopes(frame)
        .expect("scopes")
        .into_iter()
        .find(|scope| scope.name == "Locals")
        .expect("locals")
        .variables_reference;
    assert_eq!(
        pages
            .variables(locals, 0, 2)
            .expect_err("variable page limit")
            .kind,
        DebugErrorKind::InspectionLimit
    );

    let limits = DebugInspectionLimits {
        max_handles: 2,
        ..DebugInspectionLimits::default()
    };
    let mut handles =
        DebugSession::with_args_and_limits(inspection_executable(), Vec::new(), limits)
            .expect("handle-bounded session");
    stopped(handles.step_into().expect("initialize locals"));
    let frame = handles.stack(0, 1).expect("frame").items[0].id;
    let locals = handles
        .scopes(frame)
        .expect("scopes")
        .into_iter()
        .find(|scope| scope.name == "Locals")
        .expect("locals")
        .variables_reference;
    assert_eq!(
        handles
            .variables(locals, 0, 10)
            .expect_err("child handle limit")
            .kind,
        DebugErrorKind::InspectionLimit
    );

    let limits = DebugInspectionLimits {
        max_output_bytes: 1,
        ..DebugInspectionLimits::default()
    };
    let mut output =
        DebugSession::with_args_and_limits(inspection_executable(), Vec::new(), limits)
            .expect("output-bounded session");
    stopped(output.step_into().expect("initialize locals"));
    let frame = output.stack(0, 1).expect("frame").items[0].id;
    let locals = output
        .scopes(frame)
        .expect("scopes")
        .into_iter()
        .find(|scope| scope.name == "Locals")
        .expect("locals")
        .variables_reference;
    let variables = output.variables(locals, 0, 10).expect("bounded output");
    assert!(variables.items.is_empty());
    assert_eq!(variables.total, 4);
}
