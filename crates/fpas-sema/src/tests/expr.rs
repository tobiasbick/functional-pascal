use super::{check_errors, check_ok};
use crate::analyze_with_types;

// ── Literals ────────────────────────────────────────────────────

#[test]
fn integer_literal() {
    check_ok("program T; var X: integer := 42; begin end.");
}

#[test]
fn real_literal() {
    check_ok("program T; var X: real := 3.14; begin end.");
}

#[test]
fn string_literal() {
    check_ok("program T; var X: string := 'hello'; begin end.");
}

#[test]
fn single_character_string_literal_defaults_to_string() {
    let (program, parse_errors) = fpas_parser::parse("program T; var X: string := 'A'; begin end.");
    assert!(parse_errors.is_empty(), "{parse_errors:#?}");

    let value = match &program.declarations[0] {
        fpas_parser::Decl::Var(var_def) => &var_def.value,
        other => panic!("expected variable declaration, got {other:?}"),
    };

    let (errors, types, _method_calls) = analyze_with_types(&program);
    assert!(errors.is_empty(), "{errors:#?}");

    let key = crate::expr_lookup_key(value);
    assert_eq!(types.get(&key), Some(&crate::Ty::String));
}

#[test]
fn bool_literal() {
    check_ok("program T; var X: boolean := true; begin end.");
}

// ── Arithmetic ──────────────────────────────────────────────────

#[test]
fn add_integers() {
    check_ok("program T; var X: integer := 1 + 2; begin end.");
}

#[test]
fn add_reals() {
    check_ok("program T; var X: real := 1.0 + 2.0; begin end.");
}

#[test]
fn analyze_with_types_records_expression_types() {
    let (program, parse_errors) =
        fpas_parser::parse("program T; var X: real := 1.0 + 2.0; begin end.");
    assert!(parse_errors.is_empty());
    let (errs, map, _method_calls) = analyze_with_types(&program);
    assert!(errs.is_empty(), "{errs:?}");
    assert!(
        map.len() >= 3,
        "expected types for literals and binary expression"
    );
}

#[test]
fn mixed_numeric() {
    // integer + real → real (promotion)
    check_ok("program T; var X: real := 1 + 2.0; begin end.");
}

#[test]
fn add_strings() {
    check_ok("program T; var X: string := 'a' + 'b'; begin end.");
}

#[test]
fn add_type_error() {
    check_errors("program T; var X: integer := 1 + true; begin end.");
}

#[test]
fn int_div_valid() {
    check_ok("program T; var X: integer := 10 div 3; begin end.");
}

#[test]
fn int_div_with_real_error() {
    check_errors("program T; var X: integer := 10 div 3.0; begin end.");
}

#[test]
fn mod_valid() {
    check_ok("program T; var X: integer := 10 mod 3; begin end.");
}

// ── Logical ─────────────────────────────────────────────────────

#[test]
fn and_booleans() {
    check_ok("program T; var X: boolean := true and false; begin end.");
}

#[test]
fn or_booleans() {
    check_ok("program T; var X: boolean := true or false; begin end.");
}

#[test]
fn and_integers_bitwise() {
    check_ok("program T; var X: integer := 5 and 3; begin end.");
}

#[test]
fn not_boolean() {
    check_ok("program T; var X: boolean := not true; begin end.");
}

#[test]
fn negate_integer() {
    check_ok("program T; var X: integer := -42; begin end.");
}

#[test]
fn negate_non_numeric_error() {
    check_errors("program T; var X: integer := -true; begin end.");
}

// ── Comparison ──────────────────────────────────────────────────

#[test]
fn compare_integers() {
    check_ok("program T; var X: boolean := 1 < 2; begin end.");
}

#[test]
fn compare_strings() {
    check_ok("program T; var X: boolean := 'a' < 'b'; begin end.");
}

#[test]
fn equality_same_type() {
    check_ok("program T; var X: boolean := 1 = 1; begin end.");
}

#[test]
fn equality_type_mismatch() {
    check_errors("program T; var X: boolean := 1 = true; begin end.");
}

// ── Shift ───────────────────────────────────────────────────────

#[test]
fn shl_valid() {
    check_ok("program T; var X: integer := 1 shl 4; begin end.");
}

#[test]
fn shr_valid() {
    check_ok("program T; var X: integer := 16 shr 4; begin end.");
}

// ── Array literal ───────────────────────────────────────────────

#[test]
fn array_literal_valid() {
    check_ok("program T; var X: array of integer := [1, 2, 3]; begin end.");
}

#[test]
fn array_literal_mixed_types() {
    check_errors("program T; var X: array of integer := [1, 2, true]; begin end.");
}

#[test]
fn empty_array() {
    check_ok("program T; var X: array of integer := []; begin end.");
}

// ── Designator ──────────────────────────────────────────────────

#[test]
fn undefined_variable() {
    check_errors("program T; begin return Foo end.");
}

// ── Function call ───────────────────────────────────────────────

#[test]
fn call_function() {
    check_ok(
        "program T; \
         function Add(A: integer; B: integer): integer; \
         begin return A + B end; \
         begin var X: integer := Add(1, 2) end.",
    );
}

#[test]
fn call_wrong_arg_count() {
    check_errors(
        "program T; \
         function Add(A: integer; B: integer): integer; \
         begin return A + B end; \
         begin var X: integer := Add(1) end.",
    );
}
