use super::super::{check_errors, check_ok};

#[test]
fn case_ordinal_valid() {
    check_ok(
        "program T; begin \
         case 1 of \
           1: return; \
           2: return \
         end \
         end.",
    );
}

#[test]
fn case_data_enum_rejects_foreign_root_variant() {
    let errors = check_errors(
        "program T; \
         type Shape = enum Circle(Radius: real); Point; end; \
         type Other = enum Square(Size: real); end; \
         begin \
           var S: Shape := Shape.Point; \
           case S of \
             Other.Square(Size): return; \
             Shape.Point: return \
           end \
         end.",
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
        "program T; \
         type Inner = enum A(X: integer); end; \
         type Other = enum B(X: integer); end; \
         type Outer = enum Wrap(Value: Inner); Empty; end; \
         begin \
           var V: Outer := Outer.Empty; \
           case V of \
             Outer.Wrap(Other.B(X)): return; \
             Outer.Empty: return \
           end \
         end.",
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
        "program T; \
         type Shape = enum Circle(Radius: real); Point; end; \
         begin \
           var S: Shape := Shape.Point; \
           case S of \
             Shape.Circle('big'): return; \
             Shape.Point: return \
           end \
         end.",
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
        "program T; \
         begin \
           var O: Option of integer := None; \
           case O of \
             Ok(V): return; \
             None: return \
           end \
         end.",
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
        "program T; uses Std.Console; \
         begin \
           var R: Result of string, string := Ok('hello'); \
           case R of \
             Ok(Msg), Error(Msg): WriteLn(Msg) \
           end \
         end.",
    );
}

#[test]
fn case_result_multi_label_binding_names_are_case_insensitive() {
    check_ok(
        "program T; uses Std.Console; \
         begin \
           var R: Result of string, string := Ok('hello'); \
           case R of \
             Ok(Message), Error(message): WriteLn(Message) \
           end \
         end.",
    );
}

#[test]
fn case_result_multi_label_checks_shared_body_once() {
    let errors = check_errors(
        "program T; \
         begin \
           var R: Result of string, string := Ok('hello'); \
           case R of \
             Ok(Message), Error(message): \
               var Invalid: integer := 'not an integer' \
           end \
         end.",
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
        "program T; \
         begin \
           var R: Result of integer, string := Ok(1); \
           case R of \
             Ok(Value), Error(Value): return \
           end \
         end.",
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
        "program T; \
         begin \
           var R: Result of string, string := Ok('value'); \
           case R of \
             Ok(Value), Error(Message): return \
           end \
         end.",
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
        "program T; \
         type Pair = enum Values(Left: integer; Right: integer); end; \
         begin \
           var P: Pair := Pair.Values(1, 2); \
           case P of \
             Pair.Values(Value, value): return \
           end \
         end.",
    );
    assert!(
        errors.iter().any(|error| {
            error.code == fpas_diagnostics::codes::SEMA_DUPLICATE_DECLARATION
                && error.message.contains("Pattern binding")
        }),
        "expected duplicate pattern binding, got: {errors:#?}"
    );
}
