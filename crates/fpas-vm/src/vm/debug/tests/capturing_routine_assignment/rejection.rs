//! Rejected capturing-routine assignment leaves the destination unchanged.

use super::*;

fn preserved_identity(session: &mut DebugSession, frame: u64) -> String {
    rendered(session, call("Current", 1), frame)
}

#[test]
fn wrong_owner_stale_peer_and_uninitialized_sources_are_atomic() {
    let mut session = DebugSession::new(compile_fixture()).expect("debug session");
    let owner = run_to(&mut session, "var MakeStop: integer := 0;");
    let stack = session.stack(0, 8).expect("stack");
    let root_frame = stack.items.last().expect("root").id;
    assert_eq!(preserved_identity(&mut session, owner), "1");
    let wrong = session
        .set_expression(&root("Current"), &name("AddBase"), Some(root_frame))
        .expect_err("root is not MakeAdder");
    assert_eq!(wrong.kind, DebugErrorKind::VariableValueType);
    assert!(
        wrong.hint.contains("enclosing function") || wrong.message.contains("lexical owner"),
        "{wrong:?}"
    );
    let owner = session.stack(0, 8).expect("unchanged").items[0].id;
    assert_eq!(preserved_identity(&mut session, owner), "1");

    session
        .set_expression(&root("Current"), &name("AddBase"), Some(owner))
        .expect("valid owner assign");
    let stale = session
        .set_expression(&root("Current"), &name("Identity"), Some(owner))
        .expect_err("expired frame");
    assert_eq!(stale.kind, DebugErrorKind::UnknownFrame);

    let mut late = DebugSession::new(compile_fixture()).expect("late session");
    let frame = run_to(&mut late, "var LateStop: integer := 0;");
    let uninitialized = late
        .set_expression(&root("Current"), &name("AddLate"), Some(frame))
        .expect_err("Offset is uninitialized");
    assert_eq!(uninitialized.kind, DebugErrorKind::UninitializedValue);
    let frame = late.stack(0, 1).expect("preserved").items[0].id;
    assert_eq!(preserved_identity(&mut late, frame), "1");
}

#[test]
fn task_graph_and_limits_are_rejected() {
    let mut held = DebugSession::new(compile_fixture()).expect("task graph session");
    let frame = run_to(&mut held, "var TaskStop: integer := 0;");
    let task = held
        .set_expression(&root("Current"), &name("IgnorePending"), Some(frame))
        .expect_err("task handle capture");
    assert_eq!(task.kind, DebugErrorKind::VariableValueType);
    assert!(task.message.contains("task"), "{task:?}");
    let frame = held.stack(0, 1).expect("preserved").items[0].id;
    assert_eq!(preserved_identity(&mut held, frame), "1");

    let mut limited = DebugSession::new(compile_fixture()).expect("limit session");
    let frame = run_to(&mut limited, "var MakeStop: integer := 0;");
    let depth = limited
        .set_expression_with_limits(
            &root("Current"),
            &name("AddBase"),
            Some(frame),
            DebugEvaluationLimits {
                max_depth: 0,
                ..DebugEvaluationLimits::default()
            },
        )
        .expect_err("depth");
    assert_eq!(depth.kind, DebugErrorKind::EvaluationLimit);
    let frame = limited.stack(0, 1).expect("preserved").items[0].id;
    assert_eq!(preserved_identity(&mut limited, frame), "1");

    let count = limited
        .set_expression_with_limits(
            &root("Current"),
            &name("AddBase"),
            Some(frame),
            DebugEvaluationLimits {
                max_detached_values: 0,
                ..DebugEvaluationLimits::default()
            },
        )
        .expect_err("values");
    assert_eq!(count.kind, DebugErrorKind::EvaluationLimit);
}

#[test]
fn signature_ambiguous_and_unknown_names_stay_actionable() {
    let mut session = DebugSession::new(compile_fixture()).expect("debug session");
    let frame = run_to(&mut session, "var RootStop: integer := 0;");
    let signature = session
        .set_expression(&root("Job"), &name("Identity"), Some(frame))
        .expect_err("procedure dest");
    assert_eq!(signature.kind, DebugErrorKind::VariableValueType);
    let ambiguous = session
        .set_expression(&root("Current"), &name("Transform"), Some(frame))
        .expect_err("ambiguous");
    assert_eq!(ambiguous.kind, DebugErrorKind::AmbiguousCallable);
    let unknown = session
        .set_expression(&root("Current"), &name("MissingRoutine"), Some(frame))
        .expect_err("unknown");
    assert_eq!(unknown.kind, DebugErrorKind::UnknownName);
    let frame = session.stack(0, 1).expect("preserved").items[0].id;
    assert_eq!(preserved_identity(&mut session, frame), "1");
}

#[test]
fn peer_task_frame_cannot_supply_captures_for_the_unselected_owner() {
    let mut session = DebugSession::new(compile_fixture()).expect("debug session");
    let worker = run_to(&mut session, "var WorkerStop: integer := 0;");
    let main = session.stack_for_task(0, 0, 1).expect("main stack").items[0].id;
    let rejected = session
        .set_expression(&root("Current"), &name("AddWorker"), Some(main))
        .expect_err("main is not Worker");
    assert_eq!(rejected.kind, DebugErrorKind::VariableValueType);
    session
        .set_expression(&root("Current"), &name("AddWorker"), Some(worker))
        .expect("selected worker owner");
    let frame = session.stack_for_task(1, 0, 1).expect("fresh worker").items[0].id;
    assert_eq!(rendered(&mut session, call("Current", 1), frame), "9");
}
