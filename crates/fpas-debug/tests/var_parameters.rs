//! `var` parameters in debugger inspection: writes are visible before a runtime failure.
//!
//! Documentation: `docs/pascal/language/functions/var-parameters.md`

#![allow(
    clippy::expect_used,
    clippy::panic,
    reason = "integration fixtures keep compiler and debugger failures local"
)]

use fpas_vm::{DebugExpression, DebugRunResult, DebugSession};

fn compile(source: &str) -> fpas_bytecode::VerifiedExecutable {
    let (program, diagnostics) = fpas_parser::parse(source);
    assert!(diagnostics.is_empty(), "parse diagnostics: {diagnostics:?}");
    fpas_compiler::compile(&program).expect("compile var-parameter fixture")
}

#[test]
fn writes_through_var_parameters_remain_visible_after_a_runtime_failure() {
    let source = "\
program VarFailure;
type Point = record
  X: integer;
end record;
procedure Fail(var Value: integer; var Item: integer; var Field: integer; Zero: integer);
begin
  Value := 42;
  Item := 7;
  Field := 9;
  Value := Value div Zero;
end procedure;
begin
  var Counter: integer := 0;
  var Items: array of integer := [0, 0];
  var P: Point := record X := 0; end;
  Fail(var Counter, var Items[1], var P.X, 0);
end.";
    let mut session = DebugSession::new(compile(source)).expect("debug session");
    assert!(matches!(
        session
            .continue_execution()
            .expect("run to the runtime failure"),
        DebugRunResult::Stopped(_)
    ));
    let frames = session.stack(0, 2).expect("stack").items;
    let callee = frames[0].id;
    let caller = frames[1].id;
    let name = |name: &str| DebugExpression::Name(name.to_string());
    for (frame, label, expression, expected) in [
        (callee, "Value", name("Value"), "42"),
        (caller, "Counter", name("Counter"), "42"),
        (
            caller,
            "Items[1]",
            DebugExpression::Index {
                base: Box::new(name("Items")),
                index: Box::new(DebugExpression::Integer(1)),
            },
            "7",
        ),
        (
            caller,
            "P.X",
            DebugExpression::Field {
                base: Box::new(name("P")),
                name: "X".to_string(),
            },
            "9",
        ),
    ] {
        let result = session
            .evaluate(&expression, Some(frame))
            .unwrap_or_else(|error| panic!("evaluate {label}: {error:?}"));
        assert_eq!(result.value, expected, "{label}");
    }
}
