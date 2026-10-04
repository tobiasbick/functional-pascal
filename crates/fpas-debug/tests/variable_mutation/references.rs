//! Debug writes keep caller-reference authority and never replace its register.

use super::*;

#[test]
fn var_parameter_debug_write_reaches_caller_and_preserves_following_source_access() {
    let mut session = session(
        r#"program T;
 var Count: integer := 1;
procedure Change(var Value: integer); begin Value := Value + 1; end procedure;
begin Change(var Count); if Count <> 78 then panic('debug var write'); end if; end program;"#,
    );
    while session_scope(&mut session, "Parameters").is_none() {
        step(&mut session);
    }
    let parameters = session_scope(&mut session, "Parameters").expect("parameters");
    assert_eq!(
        session.variables(parameters, 0, 10).expect("values").items[0].value,
        "1"
    );
    let globals = session_scope(&mut session, "Globals").expect("globals");
    let global = session
        .variables(globals, 0, 10)
        .expect("globals")
        .items
        .into_iter()
        .find(|value| value.name == "Count")
        .expect("Count");
    assert_eq!(global.value, "<unavailable>");
    let error = session
        .set_variable(globals, "Count", &DebugExpression::Integer(99))
        .expect_err("exclusive global is unavailable");
    assert_eq!(error.kind, DebugErrorKind::VariableUnavailable);
    session
        .set_variable(parameters, "Value", &DebugExpression::Integer(77))
        .expect("authorized var write");
    assert!(matches!(
        session.step_out().expect("return"),
        DebugRunResult::Stopped(_)
    ));
    assert!(matches!(
        session.continue_execution().expect("finish"),
        DebugRunResult::Terminated(_)
    ));
}

#[test]
fn var_parameter_debug_descendant_write_preserves_caller_snapshot() {
    let mut session = session(
        r#"program T;
type Box = record Item: integer; end record;
procedure Change(var Value: Box); begin Value.Item := Value.Item + 1; end procedure;
begin  var Data: Box := Box(Item := 1); const Before: Box := Data; Change(var Data); if (Data.Item <> 43) or (Before.Item <> 1) then panic('snapshot'); end if; end program;"#,
    );
    while session_scope(&mut session, "Parameters").is_none() {
        step(&mut session);
    }
    let frame = session.stack(0, 1).expect("frame").items[0].id;
    session
        .set_expression(
            &DebugAssignmentTarget {
                root: "Value".to_string(),
                selectors: vec![DebugAssignmentSelector::Field("Item".to_string())],
            },
            &DebugExpression::Integer(42),
            Some(frame),
        )
        .expect("selected authorized write");
    assert!(matches!(
        session.step_out().expect("return"),
        DebugRunResult::Stopped(_)
    ));
    assert!(matches!(
        session.continue_execution().expect("finish"),
        DebugRunResult::Terminated(_)
    ));
}
