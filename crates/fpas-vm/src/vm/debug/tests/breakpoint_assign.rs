//! Global assignment through durable location identities.

use super::*;

const SOURCE: &str = r#"program BreakpointAssign;

mutable var Flag: integer := 0;

begin
  Flag := 1;
  Flag := 2
end.
"#;

fn compile_session() -> DebugSession {
    let (program, diagnostics) = fpas_parser::parse(SOURCE);
    assert!(diagnostics.is_empty(), "parse diagnostics: {diagnostics:?}");
    DebugSession::new(fpas_compiler::compile(&program).expect("compile assign fixture"))
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

fn flag_value(session: &mut DebugSession) -> String {
    session
        .evaluate(&DebugExpression::Name("Flag".to_string()), None)
        .expect("evaluate Flag")
        .value
}

#[test]
fn assign_data_location_commits_once_and_refreshes_inspection() {
    let mut session = compile_session();
    let _ = stop_at(&mut session, "Flag := 1;");
    let globals = scope_reference(&mut session, "Globals");
    assert_eq!(flag_value(&mut session), "0");

    let assigned = session
        .assign_data_location(
            DebugDataLocationIdentity::Global { index: 0 },
            &DebugExpression::Integer(99),
        )
        .expect("assign Flag");
    assert_eq!(assigned.value, "99");
    assert_eq!(flag_value(&mut session), "99");
    assert!(
        session.variables(globals, 0, 1).is_err(),
        "inspection handles expire after a successful assign"
    );

    let globals = scope_reference(&mut session, "Globals");
    assert_eq!(
        session
            .variables(globals, 0, 1)
            .expect("refreshed globals")
            .items[0]
            .value,
        "99"
    );
}

#[test]
fn failed_assign_does_not_mutate_storage_or_handles() {
    let mut session = compile_session();
    let _ = stop_at(&mut session, "Flag := 1;");
    let globals = scope_reference(&mut session, "Globals");

    let error = session
        .assign_data_location(
            DebugDataLocationIdentity::Global { index: 0 },
            &DebugExpression::String("nope".to_string()),
        )
        .expect_err("type mismatch");
    assert_eq!(error.kind, DebugErrorKind::VariableValueType);
    assert_eq!(flag_value(&mut session), "0");
    assert_eq!(
        session
            .variables(globals, 0, 1)
            .expect("unchanged snapshot")
            .items[0]
            .value,
        "0"
    );
}

#[test]
fn frame_identity_is_not_assignable() {
    let mut session = compile_session();
    let _ = stop_at(&mut session, "Flag := 1;");
    let error = session
        .assign_data_location(
            DebugDataLocationIdentity::FrameRegister {
                task_id: 0,
                function: 0,
                register: 0,
            },
            &DebugExpression::Integer(1),
        )
        .expect_err("frame identity");
    assert_eq!(error.kind, DebugErrorKind::VariablePathUnsupported);
    assert_eq!(flag_value(&mut session), "0");
}
