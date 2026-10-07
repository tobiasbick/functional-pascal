//! `var` parameters: argument markers, validity, aliasing, and lifetime rules.
//!
//! Documentation: `docs/pascal/language/functions/var-parameters.md`

use super::{check_errors, check_ok};
use fpas_diagnostics::DiagnosticCode;
use fpas_diagnostics::codes::{
    SEMA_IMMUTABLE_ASSIGNMENT, SEMA_INVALID_VAR_ARGUMENT, SEMA_TYPE_MISMATCH,
    SEMA_VAR_ARGUMENT_ALIAS, SEMA_VAR_ARGUMENT_MARKER, SEMA_VAR_PARAMETER_ESCAPE,
};

const DECLARATIONS: &str = r#"
type Point = record
  X: integer;
  Y: integer;
end record;

var Global: integer := 0;

procedure Increase(var Value: integer);
begin
  Value := Value + 1;
end procedure;

procedure Swap(var A: integer; var B: integer);
begin
  const T: integer := A;
  A := B;
  B := T;
end procedure;

procedure Show(Value: integer);
begin
end procedure;

function Next(var Counter: integer): integer;
begin
  Counter := Counter + 1;
  return Counter;
end function;
"#;

fn program(body: &str) -> String {
    format!("program T;\nuses Std.Console, Std.Tasks;\n{DECLARATIONS}\nbegin\n{body}\nend.")
}

fn error_codes(body: &str) -> Vec<DiagnosticCode> {
    check_errors(&program(body))
        .into_iter()
        .map(|error| error.code)
        .collect()
}

#[test]
fn var_arguments_accept_variables_fields_elements_globals_and_forwarding() {
    check_ok(&program(
        r#"
  var Counter: integer := 0;
  var P: Point := record X := 1; Y := 2; end;
  var Items: array of integer := [1, 2];
  Increase(var Counter);
  Increase(var P.X);
  Increase(var Items[0]);
  Increase(var Global);
  Swap(var P.X, var Counter);
  const N: integer := Next(var Counter);
  const F: procedure(var Value: integer) := Increase;
  F(var Counter);
"#,
    ));
    check_ok(&program(
        r#"
  var Total: integer := 0;
  const Bump: procedure(var Value: integer) := procedure(var Value: integer) begin
    Increase(var Value);
  end procedure;
  Bump(var Total);
"#,
    ));
}

#[test]
fn nested_routines_may_use_enclosing_var_parameters_while_the_call_runs() {
    check_ok(
        "program T;
procedure Outer(var Total: integer);
  procedure Add();
  begin
    Total := Total + 1;
  end procedure;
begin
  Add();
end procedure;
begin
end.",
    );
}

#[test]
fn markers_must_match_parameter_modes() {
    for body in [
        "  var Counter: integer := 0;\n  Increase(Counter);",
        "  var Counter: integer := 0;\n  Show(var Counter);",
        "  Increase(1 + 2);",
        "  var Counter: integer := 0;\n  WriteLn(var Counter);",
    ] {
        assert!(
            error_codes(body).contains(&SEMA_VAR_ARGUMENT_MARKER),
            "{body}"
        );
    }
}

#[test]
fn var_arguments_must_name_writable_storage() {
    for body in [
        "  const Fixed: integer := 1;\n  Increase(var Fixed);",
        "  var D: dict of string to integer := ['a': 1];\n  Increase(var D['a']);",
        "  for I: integer := 1 to 2 do\n    Increase(var I);\n  end for;",
    ] {
        assert_eq!(error_codes(body), [SEMA_INVALID_VAR_ARGUMENT], "{body}");
    }
    let errors = check_errors(
        "program T;
procedure Increase(var Value: integer);
begin
  Value := Value + 1;
end procedure;
procedure Copy(Value: integer);
begin
  Increase(var Value);
end procedure;
begin
end.",
    );
    assert_eq!(errors[0].code, SEMA_INVALID_VAR_ARGUMENT);
}

#[test]
fn var_arguments_require_the_exact_parameter_type() {
    let errors = error_codes("  var R: real := 1.0;\n  Increase(var R);");
    assert_eq!(errors, [SEMA_TYPE_MISMATCH]);
}

#[test]
fn var_arguments_of_one_call_must_not_share_a_root() {
    for body in [
        "  var Items: array of integer := [1, 2];\n  Swap(var Items[0], var Items[1]);",
        "  var P: Point := record X := 1; Y := 2; end;\n  Swap(var P.X, var P.Y);",
        "  var Counter: integer := 0;\n  Swap(var Counter, var Counter);",
    ] {
        assert_eq!(error_codes(body), [SEMA_VAR_ARGUMENT_ALIAS], "{body}");
    }
}

#[test]
fn var_parameters_cannot_outlive_the_call() {
    let escapes = check_errors(
        "program T;
uses Std.Tasks;
procedure Increase(var Value: integer);
begin
  Value := Value + 1;
end procedure;
procedure Outer(var Total: integer);
  procedure Add();
  begin
    Total := Total + 1;
  end procedure;
begin
  const Closure: procedure() := procedure() begin
    Total := 0;
  end procedure;
  const RoutineValue: procedure() := Add;
  go Add();
end procedure;
begin
  var Counter: integer := 0;
  go Increase(var Counter);
end.",
    );
    let codes = escapes.iter().map(|error| error.code).collect::<Vec<_>>();
    assert_eq!(
        codes
            .iter()
            .filter(|code| **code == SEMA_VAR_PARAMETER_ESCAPE)
            .count(),
        4,
        "{escapes:#?}"
    );
}

#[test]
fn function_types_include_parameter_modes() {
    let errors = check_errors(
        "program T;
procedure Increase(var Value: integer);
begin
  Value := Value + 1;
end procedure;
procedure Show(Value: integer);
begin
end procedure;
begin
  const A: procedure(Value: integer) := Increase;
  const B: procedure(var Value: integer) := Show;
end.",
    );
    assert_eq!(
        errors
            .iter()
            .filter(|error| error.code == SEMA_TYPE_MISMATCH)
            .count(),
        2,
        "{errors:#?}"
    );
}

#[test]
fn receiver_calls_and_array_mutation_reject_var_receivers() {
    assert_eq!(
        error_codes("  var Counter: integer := 0;\n  Counter.Increase();"),
        [SEMA_VAR_ARGUMENT_MARKER]
    );
    let errors = check_errors(
        "program T;
uses Std.Arrays;
procedure Add(var Items: array of integer);
begin
  Push(Items, 1);
end procedure;
begin
end.",
    );
    assert_eq!(errors[0].code, SEMA_IMMUTABLE_ASSIGNMENT);
}
