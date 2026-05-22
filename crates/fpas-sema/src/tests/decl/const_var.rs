use super::{check_errors, check_ok};

#[test]
fn const_valid() {
    check_ok("program T; const Pi: real := 3.14; begin end.");
}

#[test]
fn const_can_reference_previous_const() {
    check_ok("program T; const A: integer := 40; B: integer := A + 2; begin end.");
}

#[test]
fn const_type_mismatch() {
    check_errors("program T; const X: integer := 3.14; begin end.");
}

#[test]
fn const_initializer_must_be_compile_time_known() {
    let errors = check_errors(
        "program T; \
         function FortyTwo(): integer; \
         begin return 42 end; \
         const X: integer := FortyTwo(); \
         begin end.",
    );
    assert!(
        errors
            .iter()
            .any(|error| { error.code == fpas_diagnostics::codes::SEMA_NON_CONSTANT_EXPRESSION }),
        "expected non-constant-expression diagnostic, got: {errors:#?}"
    );
}

#[test]
fn const_initializer_cannot_read_variable() {
    let errors = check_errors(
        "program T; \
         var Seed: integer := 1; \
         const X: integer := Seed; \
         begin end.",
    );
    assert!(
        errors
            .iter()
            .any(|error| { error.code == fpas_diagnostics::codes::SEMA_NON_CONSTANT_EXPRESSION }),
        "expected non-constant-expression diagnostic, got: {errors:#?}"
    );
}

#[test]
fn var_valid() {
    check_ok("program T; var X: integer := 42; begin end.");
}

#[test]
fn var_type_mismatch() {
    check_errors("program T; var X: integer := true; begin end.");
}

#[test]
fn mutable_var_valid() {
    check_ok("program T; mutable var X: integer := 0; begin end.");
}

#[test]
fn duplicate_variable() {
    check_errors("program T; var X: integer := 1; var X: integer := 2; begin end.");
}

#[test]
fn duplicate_variable_differs_only_by_case_rejected() {
    let errors = check_errors("program T; var X: integer := 1; var x: integer := 2; begin end.");
    assert!(
        errors
            .iter()
            .any(|error| error.code == fpas_diagnostics::codes::SEMA_DUPLICATE_DECLARATION),
        "expected duplicate variable error, got: {errors:#?}"
    );
}

#[test]
fn variable_names_are_case_insensitive() {
    check_ok(
        "program T; \
         var X: integer := 1; \
         var Y: integer := x; \
         begin end.",
    );
}
