use super::super::{check_errors, check_ok};

#[test]
fn case_ordinal_valid() {
    check_ok(
        r#"program T; begin case 1 of when 1: return; when 2: return; end case; end program;"#,
    );
}

#[test]
fn case_data_enum_rejects_foreign_root_variant() {
    let errors = check_errors(
        r#"program T;  type Shape = enum Circle(Radius: real); Point; end enum;  type Other = enum Square(Size: real); end enum; begin var S: Shape := Shape.Point; case S of when Other.Square(Size): return; when Shape.Point: return; end case; end program;"#,
    );
    assert!(
        errors
            .iter()
            .any(|error| error.code == fpas_diagnostics::codes::SEMA_TYPE_MISMATCH),
        "expected foreign-variant mismatch, got: {errors:#?}"
    );
}

#[test]
fn case_data_enum_rejects_foreign_nested_variant() {
    let errors = check_errors(
        r#"program T;  type Inner = enum A(X: integer); end enum;  type Other = enum B(X: integer); end enum;  type Outer = enum Wrap(Value: Inner); Empty; end enum; begin var V: Outer := Outer.Empty; case V of when Outer.Wrap(Other.B(X)): return; when Outer.Empty: return; end case; end program;"#,
    );
    assert!(
        errors
            .iter()
            .any(|error| error.code == fpas_diagnostics::codes::SEMA_TYPE_MISMATCH),
        "expected nested foreign-variant mismatch, got: {errors:#?}"
    );
}

#[test]
fn case_data_enum_pattern_literal_must_match_field_type() {
    let errors = check_errors(
        r#"program T;  type Shape = enum Circle(Radius: real); Point; end enum; begin var S: Shape := Shape.Point; case S of when Shape.Circle('big'): return; when Shape.Point: return; end case; end program;"#,
    );
    assert!(
        errors
            .iter()
            .any(|error| error.code == fpas_diagnostics::codes::SEMA_TYPE_MISMATCH),
        "expected literal type mismatch, got: {errors:#?}"
    );
}

#[test]
fn case_option_rejects_result_patterns() {
    let errors = check_errors(
        r#"program T; begin var O: Option of integer := None; case O of when Ok(V): return; when None: return; end case; end program;"#,
    );
    assert!(
        errors
            .iter()
            .any(|error| error.code == fpas_diagnostics::codes::SEMA_TYPE_MISMATCH),
        "expected Result/Option pattern mismatch, got: {errors:#?}"
    );
}

#[test]
fn case_result_multi_label_shared_binding_valid() {
    check_ok(
        r#"program T;  uses Std.Console as Console; begin var R: Result of string, string := Ok('hello'); case R of when Ok(Msg), Error(Msg): Console.WriteLn(Msg); end case; end program;"#,
    );
}

#[test]
fn case_result_multi_label_binding_names_are_case_insensitive() {
    check_ok(
        r#"program T;  uses Std.Console as Console; begin var R: Result of string, string := Ok('hello'); case R of when Ok(Message), Error(message): Console.WriteLn(Message); end case; end program;"#,
    );
}

#[test]
fn case_result_multi_label_checks_shared_body_once() {
    let errors = check_errors(
        r#"program T; begin var R: Result of string, string := Ok('hello'); case R of when Ok(Message), Error(message): var Invalid: integer := 'not an integer'; end case; end program;"#,
    );
    let body_errors = errors
        .iter()
        .filter(|error| {
            error.code == fpas_diagnostics::codes::SEMA_TYPE_MISMATCH
                && error.message.contains("variable initializer")
        })
        .count();
    assert_eq!(body_errors, 1, "expected one body diagnostic: {errors:#?}");
}

#[test]
fn case_result_multi_label_rejects_incompatible_binding_types() {
    let errors = check_errors(
        r#"program T; begin var R: Result of integer, string := Ok(1); case R of when Ok(Value), Error(Value): return; end case; end program;"#,
    );
    assert!(
        errors.iter().any(|error| {
            error.code == fpas_diagnostics::codes::SEMA_TYPE_MISMATCH
                && error.message.contains("same names with compatible types")
        }),
        "expected incompatible case bindings, got: {errors:#?}"
    );
}

#[test]
fn case_result_multi_label_rejects_different_binding_names() {
    let errors = check_errors(
        r#"program T; begin var R: Result of string, string := Ok('value'); case R of when Ok(Value), Error(Message): return; end case; end program;"#,
    );
    assert!(
        errors.iter().any(|error| {
            error.code == fpas_diagnostics::codes::SEMA_TYPE_MISMATCH
                && error.message.contains("same names with compatible types")
        }),
        "expected inconsistent case binding names, got: {errors:#?}"
    );
}

#[test]
fn case_data_enum_pattern_rejects_duplicate_binding_names() {
    let errors = check_errors(
        r#"program T;  type Pair = enum Values(Left: integer; Right: integer); end enum; begin var P: Pair := Pair.Values(1, 2); case P of when Pair.Values(Value, value): return; end case; end program;"#,
    );
    assert!(
        errors.iter().any(|error| {
            error.code == fpas_diagnostics::codes::SEMA_DUPLICATE_DECLARATION
                && error.message.contains("Pattern binding")
        }),
        "expected duplicate pattern binding, got: {errors:#?}"
    );
}
