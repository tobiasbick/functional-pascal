//! Successful Cell and EnclosingCell named-routine assignment.

use super::*;
use fpas_bytecode::{DebugCaptureKind, Value};

fn capture_kinds(canonical: &str) -> Vec<DebugCaptureKind> {
    let executable = compile_fixture();
    let image = executable.executable();
    let function = image
        .functions
        .iter()
        .find(|function| image.strings.get(function.name) == Some(canonical))
        .unwrap_or_else(|| panic!("missing {canonical}"));
    function
        .debug
        .capture_sources
        .iter()
        .map(|source| source.kind)
        .collect()
}

#[test]
fn compiled_metadata_records_cell_enclosing_and_mixed_captures() {
    assert_eq!(
        capture_kinds("mutating.addcell"),
        vec![DebugCaptureKind::Cell]
    );
    assert_eq!(
        capture_kinds("mix.mixboth"),
        vec![DebugCaptureKind::Cell, DebugCaptureKind::Value]
    );
    assert_eq!(
        capture_kinds("outercell.mid.addenclosed"),
        vec![DebugCaptureKind::EnclosingCell]
    );
}

#[test]
fn direct_cell_capture_shares_the_original_handle_and_is_task_owned() {
    let mut session = DebugSession::new(compile_fixture()).expect("debug session");
    let frame = run_to(&mut session, "var CellStop: integer := 0;");
    let original = runtime(&session, &name("Original"), frame);
    let updated = session
        .set_expression(&root("Current"), &name("AddCell"), Some(frame))
        .expect("direct cell");
    assert_eq!(updated.value, "<function mutating.addcell>");
    let frame = session.stack(0, 1).expect("fresh").items[0].id;
    let assigned = runtime(&session, &name("Current"), frame);
    let original_fn = as_function(&original);
    let assigned_fn = as_function(&assigned);
    assert!(assigned_fn.task_bound);
    assert_eq!(assigned_fn.owner_task, Some(session.last_stop().task_id));
    assert_eq!(original_fn.captures.len(), 1);
    assert_eq!(assigned_fn.captures.len(), 1);
    assert!(
        std::sync::Arc::ptr_eq(
            cell_arc(&original_fn.captures[0]),
            cell_arc(&assigned_fn.captures[0])
        ),
        "debugger construction must clone the existing cell Arc"
    );
    assert!(
        !std::ptr::eq(&**original_fn, &**assigned_fn),
        "assignment constructs a new function value"
    );
}

#[test]
fn mixed_value_and_cell_captures_keep_abi_order() {
    let mut session = DebugSession::new(compile_fixture()).expect("debug session");
    let frame = run_to(&mut session, "var MixStop: integer := 0;");
    session
        .set_expression(&root("Current"), &name("MixBoth"), Some(frame))
        .expect("mixed captures");
    let frame = session.stack(0, 1).expect("fresh").items[0].id;
    let assigned_value = runtime(&session, &name("Current"), frame);
    let assigned = as_function(&assigned_value);
    assert!(assigned.task_bound);
    assert_eq!(assigned.captures.len(), 2);
    assert!(
        matches!(assigned.captures[0], Value::Cell(_)),
        "first capture is the Cell handle: {:?}",
        assigned.captures[0]
    );
    assert!(
        matches!(assigned.captures[1], Value::Integer(4)),
        "second capture is the immutable Base value: {:?}",
        assigned.captures[1]
    );
}

#[test]
fn enclosing_cell_capture_reuses_the_transitive_handle() {
    let mut session = DebugSession::new(compile_fixture()).expect("debug session");
    let frame = run_to(&mut session, "var EnclosingStop: integer := 0;");
    let original = runtime(&session, &name("Original"), frame);
    session
        .set_expression(&root("Current"), &name("AddEnclosed"), Some(frame))
        .expect("enclosing cell");
    let frame = session.stack(0, 1).expect("fresh").items[0].id;
    let assigned = runtime(&session, &name("Current"), frame);
    assert!(std::sync::Arc::ptr_eq(
        cell_arc(&as_function(&original).captures[0]),
        cell_arc(&as_function(&assigned).captures[0])
    ));
}

#[test]
fn continuation_observes_shared_writes_through_original_and_assigned() {
    let mut session = DebugSession::new(compile_fixture()).expect("debug session");
    let frame = run_to(&mut session, "var CellStop: integer := 0;");
    session
        .set_expression(&root("Current"), &name("AddCell"), Some(frame))
        .expect("assign AddCell");
    let result = session.continue_execution().expect("continue after assign");
    match result {
        DebugRunResult::Terminated(_) | DebugRunResult::Stopped(_) => {}
    }
    let lines = session.output().lines;
    assert!(
        lines.len() >= 2,
        "expected Mutating WriteLn pair, got {lines:?}"
    );
    assert_eq!(lines[0], "12");
    assert_eq!(lines[1], "13");
}

#[test]
fn selected_recursive_activation_supplies_its_own_cell() {
    let mut inner = DebugSession::new(compile_fixture()).expect("inner walk");
    let frame = run_to_hit(&mut inner, "var WalkStop: integer := 0;", 3);
    inner
        .set_expression(&root("Current"), &name("AddAcc"), Some(frame))
        .expect("innermost Acc");
    let frame = inner.stack(0, 1).expect("fresh inner").items[0].id;
    assert_eq!(rendered(&mut inner, call("Current", 0), frame), "1");

    let mut outer = DebugSession::new(compile_fixture()).expect("outer walk");
    let _ = run_to_hit(&mut outer, "var WalkStop: integer := 0;", 3);
    let outer_frame = outer.stack(0, 8).expect("walk stack").items[2].id;
    outer
        .set_expression(&root("Current"), &name("AddAcc"), Some(outer_frame))
        .expect("outer Acc");
    let selected = outer.stack(0, 8).expect("fresh outer").items[2].id;
    assert_eq!(rendered(&mut outer, call("Current", 0), selected), "3");
}

#[test]
fn shadowed_local_cannot_replace_the_owner_cell() {
    let mut session = DebugSession::new(compile_fixture()).expect("debug session");
    let frame = run_to(&mut session, "var ShadowStop: integer := 0;");
    session
        .set_expression(&root("Current"), &name("AddOffset"), Some(frame))
        .expect("owner Offset cell");
    let frame = session.stack(0, 1).expect("fresh").items[0].id;
    assert_eq!(rendered(&mut session, call("Current", 1), frame), "9");
}

#[test]
fn uninitialized_local_and_mutable_parameter_registers_accept_task_owned_routines() {
    let mut pending = DebugSession::new(compile_fixture()).expect("pending dest");
    let frame = run_to(&mut pending, "var PendingStop: integer := 0;");
    let locals = scope_reference(&mut pending, "Locals");
    assert_eq!(
        named(
            &pending.variables(locals, 0, 30).expect("locals").items,
            "Current"
        )
        .value,
        "<uninitialized>"
    );
    assert_eq!(
        pending
            .set_expression(&root("Current"), &name("AddPending"), Some(frame))
            .expect("init local")
            .value,
        "<function pendingdest.addpending>"
    );

    let mut apply = DebugSession::new(compile_fixture()).expect("parameter dest");
    let frame = run_to(&mut apply, "var ParamStop: integer := 0");
    apply
        .set_expression(&root("Current"), &name("AddParam"), Some(frame))
        .expect("mutable parameter");
    let frame = apply.stack(0, 1).expect("fresh").items[0].id;
    let assigned_value = runtime(&apply, &name("Current"), frame);
    let assigned = as_function(&assigned_value);
    assert!(assigned.task_bound);
    assert_eq!(assigned.name, "apply.addparam");
}

#[test]
fn copying_the_constructed_task_owned_function_stays_rejected() {
    let mut session = DebugSession::new(compile_fixture()).expect("debug session");
    let frame = run_to(&mut session, "var CellStop: integer := 0;");
    session
        .set_expression(&root("Current"), &name("AddCell"), Some(frame))
        .expect("construct");
    let frame = session.stack(0, 1).expect("fresh").items[0].id;
    let copied = session
        .set_expression(&root("Copy"), &name("Current"), Some(frame))
        .expect_err("copy task-bound");
    assert_eq!(copied.kind, DebugErrorKind::VariableValueType);
    assert!(copied.message.contains("task-bound"), "{copied:?}");
}
