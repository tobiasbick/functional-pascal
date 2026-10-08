//! Writable intrinsic receivers share `var` argument storage and lifetime rules.

use super::super::{check_errors, check_ok};
use fpas_diagnostics::codes::{
    SEMA_INVALID_VAR_ARGUMENT, SEMA_VAR_ARGUMENT_MARKER, SEMA_VAR_PARAMETER_ESCAPE,
};

fn program(body: &str) -> String {
    format!("program T;  begin {body} end.")
}

#[test]
fn writable_receivers_accept_fields_elements_and_forwarded_parameters() {
    check_ok(
        r#"program T;

type Holder = record Items: array of integer; end record;
procedure Change(var H: Holder; var Rows: array of array of integer);
begin
  H.Items.Push(3);
  Rows[0].Push(4);
  const Last: integer := H.Items.Pop();
end procedure;
procedure Add(var Items: array of integer);
begin
  Items.Push(5);
  const Last: integer := Items.Pop();
  Items.Push(Last);
end procedure;
begin
  var H: Holder := record Items := [1]; end;
  var Rows: array of array of integer := [[2]];
  Change(var H, var Rows);
  Add(var H.Items);
end."#,
    );
}

#[test]
fn writable_receivers_reject_readonly_and_computed_storage() {
    for body in [
        "const A: array of integer := [1]; A.Push(2);",
        "const A: array of integer := [1]; const N: integer := A.Pop();",
        "var D: dict of string to array of integer := ['a': [1]]; D['a'].Push(2);",
        "[1].Push(2);",
        "var A: array of integer := [1]; (A).Push(2);",
        "for A: array of integer in [[1]] do A.Push(2); end for;",
    ] {
        let errors = check_errors(&program(body));
        assert!(
            errors.iter().any(|e| e.code == SEMA_INVALID_VAR_ARGUMENT),
            "{body}: {errors:#?}"
        );
    }
    let errors = check_errors(
        "program T;  procedure Add(Items: array of integer); begin Items.Push(1); end procedure; begin end.",
    );
    assert!(
        errors.iter().any(|e| e.code == SEMA_INVALID_VAR_ARGUMENT),
        "{errors:#?}"
    );
    let errors = check_errors(
        "program T;  function Make(): array of integer; begin return [1]; end function; begin Make().Push(2); end.",
    );
    assert!(
        errors.iter().any(|e| e.code == SEMA_INVALID_VAR_ARGUMENT),
        "{errors:#?}"
    );
}

#[test]
fn written_arguments_keep_var_markers_and_readonly_value_mode() {
    check_ok(&program(
        "var A: array of integer := [1]; A.Push(Value := 2); const N: integer := A.Pop(); A.Push(A[0]);",
    ));
    for body in [
        "var A: array of integer := [1]; var N: integer := 2; A.Push(var N);",
        "var A: array of integer := [1]; var N: integer := 2; A.Push(Value := var N);",
    ] {
        let errors = check_errors(&program(body));
        assert!(
            errors.iter().any(|e| e.code == SEMA_VAR_ARGUMENT_MARKER),
            "{body}: {errors:#?}"
        );
    }
}

#[test]
fn go_cannot_receive_implicit_writable_storage() {
    for body in [
        "go A.Push(2);",
        "go A.Pop();",
        "const T: task of integer := go A.Pop();",
        "const T: task := go A.Push(2);",
        "go Push(var A, 2);",
    ] {
        let errors = check_errors(&program(&format!("var A: array of integer := [1]; {body}")));
        assert_eq!(
            errors
                .iter()
                .filter(|e| e.code == SEMA_VAR_PARAMETER_ESCAPE)
                .count(),
            1,
            "{body}: {errors:#?}"
        );
    }
}

#[test]
fn closure_lifetime_rules_apply_to_unmarked_receivers() {
    let errors = check_errors(
        "program T;  procedure Add(var A: array of integer); begin const F: procedure() := procedure() begin A.Push(1); end procedure; end procedure; begin end.",
    );
    assert!(
        errors.iter().any(|e| e.code == SEMA_VAR_PARAMETER_ESCAPE),
        "{errors:#?}"
    );
    check_ok(
        "program T;  procedure Add(var A: array of integer); procedure Local(); begin A.Push(1); end procedure; begin Local(); end procedure; begin end.",
    );
}
