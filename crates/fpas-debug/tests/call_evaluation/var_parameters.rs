//! Unsupported reference calls must fail before the detached callee starts.

use super::*;

fn rejects_var_call(source: &str, expression: DebugExpression) {
    let mut session = DebugSession::new(compile(source)).expect("debug session");
    let before = session.last_stop().clone();
    let failure = session
        .evaluate(&expression, None)
        .expect_err("missing explicit var argument must be rejected");
    assert_eq!(failure.kind, DebugErrorKind::EvaluationType, "{failure:?}");
    assert!(failure.message.contains("`var` parameter"), "{failure:?}");
    assert!(failure.hint.contains("explicitly"), "{failure:?}");
    assert_eq!(session.last_stop(), &before);
}

#[test]
fn var_parameter_is_rejected_even_when_the_callee_does_not_use_it() {
    rejects_var_call(
        "program DebugVarUnused;
function Ignore(var Item: integer): integer;
begin
  return 42;
end function;
begin end.",
        call("Ignore", vec![DebugExpression::Integer(1)]),
    );
}

#[test]
fn var_parameter_rejection_precedes_a_callee_body_failure() {
    rejects_var_call(
        "program DebugVarPanic;
function FailFirst(var Item: integer): integer;
begin
  panic('callee must not execute');
  return Item;
end function;
begin end.",
        call("FailFirst", vec![DebugExpression::Integer(1)]),
    );
}

#[test]
fn var_procedure_is_rejected_before_its_body_runs() {
    rejects_var_call(
        "program DebugVarProcedure;
procedure FailFirst(var Item: integer);
begin
  panic('callee must not execute');
end procedure;
begin end.",
        call("FailFirst", vec![DebugExpression::Integer(1)]),
    );
}

#[test]
fn record_method_var_parameter_is_rejected_before_its_body_runs() {
    rejects_var_call(
        "program DebugVarMethod;
type Box = record
  Value: integer;
  function FailFirst(Self: Box; var Item: integer): integer;
  begin
    panic('callee must not execute');
    return Self.Value + Item;
  end function;
end record;
begin end.",
        DebugExpression::MethodCall {
            receiver: Box::new(DebugExpression::Record {
                name: "Box".into(),
                fields: vec![("Value".to_string(), DebugExpression::Integer(2))],
            }),
            name: "FailFirst".to_string(),
            arguments: vec![DebugExpression::Integer(1)],
        },
    );
}

#[test]
fn visible_function_value_with_var_parameter_is_rejected_before_execution() {
    let source = "program DebugVarValue;
function FailFirst(var Item: integer): integer;
begin
  panic('callee must not execute');
  return Item;
end function;
begin
  const Handler: function(var Item: integer): integer := FailFirst;
  const Marker: integer := 0;
end.";
    let mut session = DebugSession::new(compile(source)).expect("debug session");
    let breakpoint = session
        .set_breakpoint(SourceBreakpoint {
            source: "<memory>".to_string(),
            line: 9,
            column: None,
        })
        .expect("set function-value breakpoint");
    assert!(breakpoint.is_verified(), "{breakpoint:?}");
    assert!(matches!(
        session
            .continue_execution()
            .expect("reach function-value stop"),
        DebugRunResult::Stopped(_)
    ));
    let frame = session.stack(0, 1).expect("stack").items[0].id;
    let before = session.last_stop().clone();
    let failure = session
        .evaluate(
            &call("Handler", vec![DebugExpression::Integer(1)]),
            Some(frame),
        )
        .expect_err("var function value must be rejected");
    assert_eq!(failure.kind, DebugErrorKind::EvaluationType, "{failure:?}");
    assert!(failure.message.contains("`var` parameter"), "{failure:?}");
    assert_eq!(session.last_stop(), &before);
}

#[test]
fn bound_method_value_with_var_parameter_is_rejected_before_execution() {
    let source = "program DebugVarBound;
type Box = record
  Value: integer;
  function FailFirst(Self: Box; var Item: integer): integer;
  begin
    panic('callee must not execute');
    return Self.Value + Item;
  end function;
end record;
begin
  const Holder: Box := Box(Value := 2);
  const Handler: function(var Item: integer): integer := Holder.FailFirst;
  const Marker: integer := 0;
end.";
    let mut session = DebugSession::new(compile(source)).expect("debug session");
    let line = source
        .lines()
        .position(|line| line.contains("const Marker"))
        .expect("marker line")
        + 1;
    let breakpoint = session
        .set_breakpoint(SourceBreakpoint {
            source: "<memory>".to_string(),
            line: u32::try_from(line).expect("line"),
            column: None,
        })
        .expect("set bound-method breakpoint");
    assert!(breakpoint.is_verified(), "{breakpoint:?}");
    assert!(matches!(
        session
            .continue_execution()
            .expect("reach bound-method stop"),
        DebugRunResult::Stopped(_)
    ));
    let frame = session.stack(0, 1).expect("stack").items[0].id;
    let before = session.last_stop().clone();
    let failure = session
        .evaluate(
            &call("Handler", vec![DebugExpression::Integer(1)]),
            Some(frame),
        )
        .expect_err("var bound method must be rejected");
    assert_eq!(failure.kind, DebugErrorKind::EvaluationType, "{failure:?}");
    assert!(failure.message.contains("`var` parameter"), "{failure:?}");
    assert_eq!(session.last_stop(), &before);
}
