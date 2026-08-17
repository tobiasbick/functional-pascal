//! Exact lifetimes for globals, live frames, and unregistered capture cells.

use super::*;
use crate::vm::debug::{
    DebugDataLocationIdentity, DebugDataLocationKind, DebugDataLocationLifetime,
};

const SOURCE: &str = r#"program DataLocation;

mutable var Flag: integer := 0;

procedure Inner();
begin
  mutable var Nested: integer := 1;
  Nested := Nested + Flag
end;

begin
  Flag := 1;
  Inner();
  Flag := 2
end.
"#;

fn compile_session() -> DebugSession {
    let (program, diagnostics) = fpas_parser::parse(SOURCE);
    assert!(diagnostics.is_empty(), "parse diagnostics: {diagnostics:?}");
    DebugSession::new(fpas_compiler::compile(&program).expect("compile data-location fixture"))
        .expect("debug session")
}

fn line(needle: &str) -> u32 {
    u32::try_from(
        SOURCE
            .lines()
            .position(|line| line.contains(needle))
            .expect("marker")
            + 1,
    )
    .expect("line")
}

fn stop_at(session: &mut DebugSession, needle: &str) -> u64 {
    session
        .set_breakpoint(SourceBreakpoint {
            source: "<memory>".to_string(),
            line: line(needle),
            column: None,
        })
        .expect("breakpoint");
    let _ = stopped(session.continue_execution().expect("run to marker"));
    session.stack(0, 1).expect("frame").items[0].id
}

fn scope_reference(session: &mut DebugSession, scope_name: &str) -> u64 {
    let frame = session.stack(0, 1).expect("stack").items[0].id;
    session
        .scopes(frame)
        .expect("scopes")
        .into_iter()
        .find(|scope| scope.name == scope_name)
        .expect("requested scope")
        .variables_reference
}

#[test]
fn global_identity_survives_continue_with_the_same_slot() {
    let mut session = compile_session();
    let _ = stop_at(&mut session, "Flag := 1;");
    let globals = scope_reference(&mut session, "Globals");
    let first = session
        .describe_data_location(globals, "Flag")
        .expect("describe global");
    assert_eq!(first.kind, DebugDataLocationKind::Global);
    assert_eq!(first.lifetime, DebugDataLocationLifetime::Executable);
    assert!(!first.descendant);
    let DebugDataLocationIdentity::Global { index } = first.identity.expect("global identity")
    else {
        panic!("expected global identity: {first:?}");
    };
    assert!(session.data_location_is_live(&first).expect("live global"));

    let _ = stop_at(&mut session, "Flag := 2");
    assert!(
        session
            .data_location_is_live(&first)
            .expect("global survives continue")
    );
    let globals = scope_reference(&mut session, "Globals");
    let second = session
        .describe_data_location(globals, "Flag")
        .expect("describe after continue");
    assert_eq!(second.identity, first.identity);
    assert_eq!(index, 0);
    assert_eq!(
        session
            .describe_data_location(globals, "Flag")
            .expect("repeat describe")
            .identity,
        first.identity
    );
}

#[test]
fn frame_register_identity_expires_when_the_activation_returns() {
    let mut session = compile_session();
    let _ = stop_at(&mut session, "Nested := Nested + Flag");
    let locals = scope_reference(&mut session, "Locals");
    let nested = session
        .describe_data_location(locals, "Nested")
        .expect("describe nested");
    assert_eq!(nested.kind, DebugDataLocationKind::FrameRegister);
    assert_eq!(nested.lifetime, DebugDataLocationLifetime::LiveFrame);
    let DebugDataLocationIdentity::FrameRegister {
        task_id,
        function,
        register,
    } = nested.identity.expect("frame identity")
    else {
        panic!("expected frame identity: {nested:?}");
    };
    assert_eq!(task_id, 0);
    assert!(function > 0);
    assert!(session.data_location_is_live(&nested).expect("live nested"));

    let _ = stop_at(&mut session, "Flag := 2");
    assert!(
        !session
            .data_location_is_live(&nested)
            .expect("nested returned")
    );
    assert_eq!(
        session
            .describe_data_location(locals, "Nested")
            .expect_err("expired handle")
            .kind,
        DebugErrorKind::VariableTargetExpired
    );
    let _ = register;
}

#[test]
fn capture_cell_locations_stay_unregistered_and_reject_task_bound_destinations() {
    let mut session = DebugSession::new(super::function_value_assignment::assignment_executable())
        .expect("debug session");
    super::function_value_assignment::stop_with_functions(&mut session);
    let captures = scope_reference(&mut session, "Captures");
    let location = session
        .describe_data_location(captures, "CellSlot")
        .expect("describe cell");
    assert_eq!(location.kind, DebugDataLocationKind::ClosureCell);
    assert_eq!(
        location.lifetime,
        DebugDataLocationLifetime::UnregisteredAlias
    );
    assert_eq!(location.identity, None);
    assert!(
        !session
            .data_location_is_live(&location)
            .expect("cells are not watchpoint identities")
    );

    let before = session
        .variables(captures, 0, 10)
        .expect("before")
        .items
        .into_iter()
        .find(|variable| variable.name == "CellSlot")
        .expect("cell")
        .value;
    let rejected = session
        .set_variable(
            captures,
            "CellSlot",
            &DebugExpression::Name("Bound".to_string()),
        )
        .expect_err("task-bound capture-cell destination");
    assert_eq!(rejected.kind, DebugErrorKind::VariableValueType);
    assert!(
        rejected.message.contains("capture-cell") || rejected.hint.contains("captured mutable"),
        "{rejected:?}"
    );
    let after = session
        .variables(captures, 0, 10)
        .expect("unchanged generation")
        .items
        .into_iter()
        .find(|variable| variable.name == "CellSlot")
        .expect("cell")
        .value;
    assert_eq!(after, before);
}
