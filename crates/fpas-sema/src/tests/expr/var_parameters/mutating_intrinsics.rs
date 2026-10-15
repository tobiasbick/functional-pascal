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
  H.Items.PushInPlace(3);
  Rows[0].PushInPlace(4);
  const Last: integer := H.Items.PopInPlace();
end procedure;
procedure Add(var Items: array of integer);
begin
  Items.PushInPlace(5);
  const Last: integer := Items.PopInPlace();
  Items.PushInPlace(Last);
end procedure;
begin
  var H: Holder := Holder( Items := [1] );
  var Rows: array of array of integer := [[2]];
  Change(var H, var Rows);
  Add(var H.Items);
end."#,
    );
}

#[test]
fn writable_receivers_reject_readonly_and_computed_storage() {
    for body in [
        "const A: array of integer := [1]; A.SortInPlace();",
        "const A := [1]; A.SortByInPlace(function(X: integer): integer begin return X; end function);",
        "const S: string := 'abc'; S.ReverseInPlace();",
        "const D: dict of string to integer := ['a': 1]; D.RemoveInPlace('a');",
        "var D: dict of string to string := ['a': 'x']; D['a'].TrimInPlace();",
        "const A: array of integer := [1]; A.PushInPlace(2);",
        "const A: array of integer := [1]; const N: integer := A.PopInPlace();",
        "var D: dict of string to array of integer := ['a': [1]]; D['a'].PushInPlace(2);",
        "[1].PushInPlace(2);",
        "var A: array of integer := [1]; (A).PushInPlace(2);",
        "for A: array of integer in [[1]] do A.PushInPlace(2); end for;",
    ] {
        let errors = check_errors(&program(body));
        assert!(
            errors.iter().any(|e| e.code == SEMA_INVALID_VAR_ARGUMENT),
            "{body}: {errors:#?}"
        );
    }
    let errors = check_errors(
        "program T;  procedure Add(Items: array of integer); begin Items.PushInPlace(1); end procedure; begin end.",
    );
    assert!(
        errors.iter().any(|e| e.code == SEMA_INVALID_VAR_ARGUMENT),
        "{errors:#?}"
    );
    let errors = check_errors(
        "program T;  function Make(): array of integer; begin return [1]; end function; begin Make().PushInPlace(2); end.",
    );
    assert!(
        errors.iter().any(|e| e.code == SEMA_INVALID_VAR_ARGUMENT),
        "{errors:#?}"
    );
}

#[test]
fn written_arguments_keep_var_markers_and_readonly_value_mode() {
    check_ok(&program(
        "var A: array of integer := [1]; A.PushInPlace(Value := 2); const N: integer := A.PopInPlace(); A.PushInPlace(A[0]);",
    ));
    for body in [
        "var S: string := 'abc'; var N: integer := 0; S.SliceInPlace(Start := var N, Len := 1);",
        "var A: array of integer := [1]; var N: integer := 2; A.PushInPlace(var N);",
        "var A: array of integer := [1]; var N: integer := 2; A.PushInPlace(Value := var N);",
    ] {
        let errors = check_errors(&program(body));
        assert!(
            errors.iter().any(|e| e.code == SEMA_VAR_ARGUMENT_MARKER),
            "{body}: {errors:#?}"
        );
    }
}

#[test]
fn in_place_callbacks_preserve_types_and_mutators_cannot_chain() {
    for body in [
        "var A := [1]; A.MapInPlace(function(X: integer): string begin return 'x'; end function);",
        "var A := [1]; A.FlatMapInPlace(function(X: integer): array of string begin return ['x']; end function);",
        "var D: dict of string to integer := ['a': 1]; D.MapInPlace(function(X: integer): string begin return 'x'; end function);",
        "var A := [1]; const N := A.SortInPlace().Length();",
        "var A := [1]; const N := A.SortByInPlace(function(X: integer): integer begin return X; end function).Length();",
        "var S := 'abc'; const N := S.TrimInPlace().Length();",
    ] {
        let errors = check_errors(&program(body));
        assert!(
            errors
                .iter()
                .any(|e| e.code == fpas_diagnostics::codes::SEMA_TYPE_MISMATCH),
            "{body}: {errors:#?}"
        );
    }
    check_ok(&program(
        "const A := [1]; const B := A.Map(function(X: integer): string begin return 'x'; end function); const P: ArrayPopResult of string := B.Pop();",
    ));
}

#[test]
fn go_cannot_receive_implicit_writable_storage() {
    for body in [
        "go A.PushInPlace(2);",
        "go A.SortInPlace();",
        "go A.SortByInPlace(function(X: integer): integer begin return X; end function);",
        "var S := 'abc'; discard go S.TrimInPlace();",
        "var D: dict of string to integer := ['a': 1]; discard go D.RemoveInPlace('a');",
        "go A.PopInPlace();",
        "const T: task of integer := go A.PopInPlace();",
        "const T: task := go A.PushInPlace(2);",
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
        "program T;  procedure Add(var A: array of integer); begin const F: procedure() := procedure() begin A.PushInPlace(1); end procedure; end procedure; begin end.",
    );
    assert!(
        errors.iter().any(|e| e.code == SEMA_VAR_PARAMETER_ESCAPE),
        "{errors:#?}"
    );
    check_ok(
        "program T;  procedure Add(var A: array of integer); procedure Local(); begin A.PushInPlace(1); end procedure; begin Local(); end procedure; begin end.",
    );
}
