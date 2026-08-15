//! Rejected cell-capturing assignment leaves destination and cells unchanged.

use super::*;

fn preserved_identity(session: &mut DebugSession, frame: u64) -> String {
    rendered(session, call("Current", 1), frame)
}

#[test]
fn wrong_owner_stale_and_uninitialized_cell_sources_are_atomic() {
    let mut session = DebugSession::new(compile_fixture()).expect("debug session");
    let owner = run_to(&mut session, "var CellStop: integer := 0;");
    let stack = session.stack(0, 8).expect("stack");
    let root_frame = stack.items.last().expect("root").id;
    assert_eq!(preserved_identity(&mut session, owner), "1");
    let wrong = session
        .set_expression(&root("Current"), &name("AddCell"), Some(root_frame))
        .expect_err("root is not Mutating");
    assert_eq!(wrong.kind, DebugErrorKind::VariableValueType);
    assert!(
        wrong.hint.contains("enclosing function") || wrong.message.contains("lexical owner"),
        "{wrong:?}"
    );
    let owner = session.stack(0, 8).expect("unchanged").items[0].id;
    assert_eq!(preserved_identity(&mut session, owner), "1");

    session
        .set_expression(&root("Current"), &name("AddCell"), Some(owner))
        .expect("valid owner assign");
    let stale = session
        .set_expression(&root("Current"), &name("Identity"), Some(owner))
        .expect_err("expired frame");
    assert_eq!(stale.kind, DebugErrorKind::UnknownFrame);

    let mut late = DebugSession::new(compile_fixture()).expect("late session");
    let frame = run_to(&mut late, "var LateStop: integer := 0;");
    let uninitialized = late
        .set_expression(&root("Current"), &name("AddLate"), Some(frame))
        .expect_err("Cell is uninitialized");
    assert_eq!(uninitialized.kind, DebugErrorKind::UninitializedValue);
    let frame = late.stack(0, 1).expect("preserved").items[0].id;
    assert_eq!(preserved_identity(&mut late, frame), "1");
}

#[test]
fn global_descendant_capture_cell_and_immutable_destinations_are_rejected() {
    let mut global = DebugSession::new(compile_fixture()).expect("global dest");
    let frame = run_to(&mut global, "var CellStop: integer := 0;");
    let rejected = global
        .set_expression(&root("Shared"), &name("AddCell"), Some(frame))
        .expect_err("global");
    assert_eq!(rejected.kind, DebugErrorKind::VariableValueType);
    assert!(
        rejected.message.contains("global") || rejected.hint.contains("Globals"),
        "{rejected:?}"
    );
    let frame = global.stack(0, 1).expect("preserved").items[0].id;
    assert_eq!(preserved_identity(&mut global, frame), "1");

    let mut boxed = DebugSession::new(compile_fixture()).expect("descendant dest");
    let frame = run_to(&mut boxed, "var BoxStop: integer := 0;");
    let descendant = boxed
        .set_expression(&field("Packed", "Item"), &name("AddBoxed"), Some(frame))
        .expect_err("descendant");
    assert_eq!(descendant.kind, DebugErrorKind::VariableValueType);
    assert!(
        descendant.message.contains("descendant") || descendant.hint.contains("complete mutable"),
        "{descendant:?}"
    );

    let mut captured = DebugSession::new(compile_fixture()).expect("capture dest");
    let frame = run_to(&mut captured, "var CaptureDestStop: integer := 0;");
    let cell_dest = captured
        .set_expression(&root("Current"), &name("AddCaptured"), Some(frame))
        .expect_err("capture cell dest");
    assert_eq!(cell_dest.kind, DebugErrorKind::VariableValueType);
    assert!(
        cell_dest.message.contains("capture-cell") || cell_dest.hint.contains("captured mutable"),
        "{cell_dest:?}"
    );

    let mut frozen = DebugSession::new(compile_fixture()).expect("immutable dest");
    let frame = run_to(&mut frozen, "var CellStop: integer := 0;");
    assert_eq!(
        frozen
            .set_expression(&root("Frozen"), &name("AddCell"), Some(frame))
            .expect_err("immutable")
            .kind,
        DebugErrorKind::VariableNotMutable
    );
}

#[test]
fn copied_task_owned_functions_cannot_escape_the_owner_frame() {
    let mut global = DebugSession::new(compile_fixture()).expect("global copy");
    let frame = run_to(&mut global, "var CellStop: integer := 0;");
    global
        .set_expression(&root("Current"), &name("AddCell"), Some(frame))
        .expect("construct source");
    let frame = global.stack(0, 1).expect("fresh").items[0].id;
    let rejected = global
        .set_expression(&root("Shared"), &name("Current"), Some(frame))
        .expect_err("global escape");
    assert_eq!(rejected.kind, DebugErrorKind::VariableValueType);
    assert!(rejected.message.contains("global"), "{rejected:?}");
    assert_eq!(rendered(&mut global, call("Shared", 1), frame), "1");

    let mut descendant = DebugSession::new(compile_fixture()).expect("descendant copy");
    let frame = run_to(&mut descendant, "var CellStop: integer := 0;");
    descendant
        .set_expression(&root("Current"), &name("AddCell"), Some(frame))
        .expect("construct source");
    let frame = descendant.stack(0, 1).expect("fresh").items[0].id;
    let rejected = descendant
        .set_expression(&field("Packed", "Item"), &name("Current"), Some(frame))
        .expect_err("descendant escape");
    assert_eq!(rejected.kind, DebugErrorKind::VariableValueType);
    assert!(rejected.message.contains("descendant"), "{rejected:?}");
    let packed = runtime(
        &descendant,
        &DebugExpression::Field {
            base: Box::new(name("Packed")),
            name: "Item".to_string(),
        },
        frame,
    );
    assert_eq!(as_function(&packed).name, "identity");

    let mut captured = DebugSession::new(compile_fixture()).expect("capture copy");
    let frame = run_to(&mut captured, "var CaptureDestStop: integer := 0;");
    captured
        .set_expression(&root("Source"), &name("AddCaptured"), Some(frame))
        .expect("construct source");
    let frame = captured.stack(0, 1).expect("fresh").items[0].id;
    let rejected = captured
        .set_expression(&root("Current"), &name("Source"), Some(frame))
        .expect_err("capture-cell escape");
    assert_eq!(rejected.kind, DebugErrorKind::VariableValueType);
    assert!(rejected.message.contains("capture-cell"), "{rejected:?}");
    let current = runtime(&captured, &name("Current"), frame);
    assert_eq!(as_function(&current).name, "identity");
}

#[test]
fn copied_task_owned_functions_reject_foreign_owner_and_stale_frame() {
    let mut foreign = DebugSession::new(compile_fixture()).expect("foreign copy");
    foreign
        .set_breakpoint(SourceBreakpoint {
            source: "<memory>".to_string(),
            line: line("var RootStop: integer := 0;"),
            column: None,
        })
        .expect("root breakpoint");
    let worker = run_to(&mut foreign, "var WorkerStop: integer := 0;");
    foreign
        .set_expression(&root("Current"), &name("AddWorker"), Some(worker))
        .expect("construct worker-owned source");
    let _ = stopped(foreign.continue_execution().expect("reach root"));
    let root_frame = foreign.stack(0, 1).expect("root frame").items[0].id;
    let rejected = foreign
        .set_expression(&root("Current"), &name("Stolen"), Some(root_frame))
        .expect_err("foreign owner");
    assert_eq!(rejected.kind, DebugErrorKind::VariableValueType);
    assert!(rejected.message.contains("belongs to task"), "{rejected:?}");
    assert_eq!(rendered(&mut foreign, call("Current", 1), root_frame), "1");

    let mut stale = DebugSession::new(compile_fixture()).expect("stale copy");
    let frame = run_to(&mut stale, "var CellStop: integer := 0;");
    stale
        .set_expression(&root("Current"), &name("AddCell"), Some(frame))
        .expect("construct source");
    assert_eq!(
        stale
            .set_expression(&root("Copy"), &name("Current"), Some(frame))
            .expect_err("stale frame")
            .kind,
        DebugErrorKind::UnknownFrame
    );
    let frame = stale.stack(0, 1).expect("preserved frame").items[0].id;
    assert_eq!(rendered(&mut stale, call("Copy", 1), frame), "1");
}

#[test]
fn task_spawn_rejects_the_constructed_function() {
    let mut session = DebugSession::new(compile_fixture()).expect("spawn session");
    let frame = run_to(&mut session, "var SpawnStop: integer := 0;");
    session
        .set_expression(&root("Current"), &name("RunSpawn"), Some(frame))
        .expect("assign task-bound procedure");
    let stop = stopped(
        session
            .continue_execution()
            .expect("go Current is a runtime failure"),
    );
    assert_eq!(stop.reason, DebugStopReason::RuntimeError);
    let diagnostic = stop.diagnostic.expect("runtime diagnostic");
    assert_eq!(
        diagnostic.code,
        fpas_diagnostics::codes::RUNTIME_INVALID_TASK
    );
    assert!(
        diagnostic.message.contains("spawn") || diagnostic.message.contains("task-bound"),
        "{}",
        diagnostic.message
    );
}

#[test]
fn foreign_task_invocation_fails_before_callee_entry() {
    let mut session = DebugSession::new(compile_fixture()).expect("worker session");
    let worker = run_to(&mut session, "var WorkerStop: integer := 0;");
    session
        .set_expression(&root("Current"), &name("AddWorker"), Some(worker))
        .expect("worker owner assign");
    let stop = stopped(
        session
            .continue_execution()
            .expect("foreign CallValue is a runtime failure"),
    );
    assert_eq!(stop.reason, DebugStopReason::RuntimeError);
    let diagnostic = stop.diagnostic.expect("runtime diagnostic");
    assert_eq!(
        diagnostic.code,
        fpas_diagnostics::codes::RUNTIME_INVALID_TASK
    );
    assert!(
        diagnostic.message.contains("foreign task"),
        "{}",
        diagnostic.message
    );
}

#[test]
fn capture_graph_limits_and_unknown_names_stay_actionable() {
    let mut limited = DebugSession::new(compile_fixture()).expect("limit session");
    let frame = run_to(&mut limited, "var MixStop: integer := 0;");
    let values = limited
        .set_expression_with_limits(
            &root("Current"),
            &name("MixBoth"),
            Some(frame),
            DebugEvaluationLimits {
                max_detached_values: 0,
                ..DebugEvaluationLimits::default()
            },
        )
        .expect_err("values");
    assert_eq!(values.kind, DebugErrorKind::EvaluationLimit);
    let frame = limited.stack(0, 1).expect("preserved").items[0].id;
    assert_eq!(preserved_identity(&mut limited, frame), "1");

    let unknown = limited
        .set_expression(&root("Current"), &name("MissingRoutine"), Some(frame))
        .expect_err("unknown");
    assert_eq!(unknown.kind, DebugErrorKind::UnknownName);
}

#[test]
fn success_expires_old_variable_references_once() {
    let mut session = DebugSession::new(compile_fixture()).expect("debug session");
    let frame = run_to(&mut session, "var CellStop: integer := 0;");
    let locals = scope_reference(&mut session, "Locals");
    session
        .set_variable(locals, "Current", &name("AddCell"))
        .expect("assign");
    assert_eq!(
        session
            .set_variable(locals, "Current", &name("AddCell"))
            .expect_err("stale handle")
            .kind,
        DebugErrorKind::VariableTargetExpired
    );
    let stale = frame;
    let frame = session.stack(0, 1).expect("fresh").items[0].id;
    assert_eq!(
        session
            .set_expression(&root("Current"), &name("AddCell"), Some(stale))
            .expect_err("stale frame")
            .kind,
        DebugErrorKind::UnknownFrame
    );
    let assigned_value = runtime(&session, &name("Current"), frame);
    let assigned = as_function(&assigned_value);
    assert!(assigned.task_bound);
}
