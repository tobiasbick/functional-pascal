//! Complete variant replacement in local copies and captured cells.

use fpas_vm::{DebugAssignmentTarget, DebugExpression, DebugRunResult, DebugSession};

fn session(source: &str) -> DebugSession {
    let (program, diagnostics) = fpas_parser::parse(source);
    assert!(diagnostics.is_empty(), "parse diagnostics: {diagnostics:?}");
    let executable = fpas_compiler::compile(&program).expect("compile variant session fixture");
    DebugSession::new(executable).expect("debug session")
}

fn session_scope(session: &mut DebugSession, name: &str) -> Option<u64> {
    let frame = session.stack(0, 1).ok()?.items.first()?.id;
    session
        .scopes(frame)
        .ok()?
        .into_iter()
        .find(|scope| scope.name == name)
        .map(|scope| scope.variables_reference)
}

fn step(session: &mut DebugSession) {
    assert!(matches!(
        session.step_into().expect("step"),
        DebugRunResult::Stopped(_)
    ));
}

fn root(name: &str) -> DebugAssignmentTarget {
    DebugAssignmentTarget {
        root: name.to_string(),
        selectors: Vec::new(),
    }
}

#[test]
fn variant_replacement_supports_parameter_copies_and_capture_cells() {
    let mut parameter = session(
        r#"program VariantParameter;

type Choice = enum
  Count(Value: integer);
  Pair(Left: integer; Right: integer);
end enum;

function ReadChoice(InitialItem: Choice): integer;
begin
   var Item: Choice := InitialItem;
  const Marker: integer := 0;
  case Item of
    when Choice.Count(const Value):
      begin
        return Value;
      end;
    when Choice.Pair(const Left, const Right):
      begin
        return Left + Right;
      end;
  end case;
end function;

begin
  const OutputValue: integer := ReadChoice(Choice.Count(1));
  const Marker: integer := OutputValue;
end program;
"#,
    );
    while session_scope(&mut parameter, "Parameters").is_none() {
        step(&mut parameter);
    }
    step(&mut parameter);
    let parameter_frame = parameter.stack(0, 1).expect("parameter copy stack").items[0].id;
    parameter
        .set_expression(
            &root("Item"),
            &DebugExpression::Call {
                callee: Box::new(DebugExpression::Callable("Choice.Pair".to_string())),
                arguments: vec![DebugExpression::Integer(2), DebugExpression::Integer(3)],
            },
            Some(parameter_frame),
        )
        .expect("replace mutable enum parameter copy");
    assert!(matches!(
        parameter
            .step_out()
            .expect("return from parameter function"),
        DebugRunResult::Stopped(_)
    ));
    let locals = session_scope(&mut parameter, "Locals").expect("caller locals");
    let values = parameter.variables(locals, 0, 10).expect("caller values");
    assert_eq!(
        values
            .items
            .iter()
            .find(|value| value.name == "OutputValue")
            .expect("parameter result")
            .value,
        "5"
    );

    let mut capture = session(
        r#"program VariantCapture;

type Choice = enum
  Count(Value: integer);
  Pair(Left: integer; Right: integer);
end enum;

function NextChoice(): function(): integer;
begin
   var Selected: Choice := Choice.Count(1);
  return function(): integer begin
    case Selected of
      when Choice.Count(const Value):
        begin
          return Value;
        end;
      when Choice.Pair(const Left, const Right):
        begin
          return Left + Right;
        end;
    end case;
  end function;
end function;

begin
  const Next: function(): integer := NextChoice();
  const First: integer := Next();
  const Marker: integer := First;
end program;
"#,
    );
    let (frame, _captures) = loop {
        if let Some(captures) = session_scope(&mut capture, "Captures") {
            break (
                capture.stack(0, 1).expect("capture stack").items[0].id,
                captures,
            );
        }
        step(&mut capture);
    };
    capture
        .set_expression(
            &root("Selected"),
            &DebugExpression::Call {
                callee: Box::new(DebugExpression::Callable("Choice.Pair".to_string())),
                arguments: vec![DebugExpression::Integer(20), DebugExpression::Integer(21)],
            },
            Some(frame),
        )
        .expect("replace captured enum");
    assert!(matches!(
        capture.step_out().expect("return from closure"),
        DebugRunResult::Stopped(_)
    ));
    let locals = session_scope(&mut capture, "Locals").expect("caller locals");
    let values = capture.variables(locals, 0, 10).expect("caller values");
    assert_eq!(
        values
            .items
            .iter()
            .find(|value| value.name == "First")
            .expect("closure result")
            .value,
        "41"
    );
}
