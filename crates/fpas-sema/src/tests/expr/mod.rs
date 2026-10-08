use super::{check_errors, check_ok};
use crate::analyze_with_types;

mod boolean;
mod bound_methods;
mod closures;
mod equality;
mod fluent;
mod named_arguments;
mod native;
mod postfix;
mod record_construction;
mod record_context;
mod record_events;
mod record_updates;
mod std_shadowing;
mod var_parameters;

// ── Literals ────────────────────────────────────────────────────

#[test]
fn integer_literal() {
    check_ok("program T; const X: integer := 42; begin end.");
}

#[test]
fn real_literal() {
    check_ok("program T; const X: real := 3.14; begin end.");
}

#[test]
fn string_literal() {
    check_ok("program T; const X: string := 'hello'; begin end.");
}

#[test]
fn single_character_string_literal_defaults_to_string() {
    let (program, parse_errors) =
        fpas_parser::parse("program T; const X: string := 'A'; begin end.");
    assert!(parse_errors.is_empty(), "{parse_errors:#?}");

    let value = match &program.declarations[0] {
        fpas_parser::Decl::Const(binding) => &binding.value,
        other => panic!("expected constant declaration, got {other:?}"),
    };

    let metadata = analyze_with_types(&program);
    assert!(metadata.errors.is_empty(), "{:#?}", metadata.errors);

    let key = crate::expr_lookup_key(value);
    assert_eq!(metadata.expr_types.get(&key), Some(&crate::Ty::String));
}

#[test]
fn bool_literal() {
    check_ok("program T; const X: boolean := true; begin end.");
}

// ── Arithmetic ──────────────────────────────────────────────────

#[test]
fn add_integers() {
    check_ok("program T; const X: integer := 1 + 2; begin end.");
}

#[test]
fn add_reals() {
    check_ok("program T; const X: real := 1.0 + 2.0; begin end.");
}

#[test]
fn analyze_with_types_records_expression_types() {
    let (program, parse_errors) =
        fpas_parser::parse("program T; const X: real := 1.0 + 2.0; begin end.");
    assert!(parse_errors.is_empty());
    let metadata = analyze_with_types(&program);
    assert!(metadata.errors.is_empty(), "{:?}", metadata.errors);
    assert!(
        metadata.expr_types.len() >= 3,
        "expected types for literals and binary expression"
    );
}

#[test]
fn mixed_numeric() {
    // integer + real → real (promotion)
    check_ok("program T; const X: real := 1 + 2.0; begin end.");
}

#[test]
fn add_strings() {
    check_ok("program T; const X: string := 'a' + 'b'; begin end.");
}

#[test]
fn add_type_error() {
    check_errors("program T; const X: integer := 1 + true; begin end.");
}

#[test]
fn int_div_valid() {
    check_ok("program T; const X: integer := 10 div 3; begin end.");
}

#[test]
fn real_div_is_real_even_for_integer_operands() {
    check_ok("program T; const X: real := (80 - 42) / 2; begin end.");
    check_errors("program T; const X: integer := (80 - 42) / 2; begin end.");
}

#[test]
fn int_div_with_real_error() {
    check_errors("program T; const X: integer := 10 div 3.0; begin end.");
}

#[test]
fn mod_valid() {
    check_ok("program T; const X: integer := 10 mod 3; begin end.");
}

// ── Logical ─────────────────────────────────────────────────────

#[test]
fn and_booleans() {
    check_ok("program T; const X: boolean := true and false; begin end.");
}

#[test]
fn or_booleans() {
    check_ok("program T; const X: boolean := true or false; begin end.");
}

#[test]
fn bit_and_integers() {
    check_ok("program T; uses Std.Bits; const X: integer := BitAnd(5, 3); begin end.");
}

#[test]
fn not_boolean() {
    check_ok("program T; const X: boolean := not true; begin end.");
}

#[test]
fn negate_integer() {
    check_ok("program T; const X: integer := -42; begin end.");
}

#[test]
fn negate_non_numeric_error() {
    check_errors("program T; const X: integer := -true; begin end.");
}

// ── Comparison ──────────────────────────────────────────────────

#[test]
fn compare_integers() {
    check_ok("program T; const X: boolean := 1 < 2; begin end.");
}

#[test]
fn compare_strings() {
    check_ok("program T; const X: boolean := 'a' < 'b'; begin end.");
}

#[test]
fn equality_same_type() {
    check_ok("program T; const X: boolean := 1 = 1; begin end.");
}

#[test]
fn equality_type_mismatch() {
    check_errors("program T; const X: boolean := 1 = true; begin end.");
}

#[test]
fn analyze_with_types_records_canonical_intrinsics_and_type_operations() {
    let (program, parse_errors) = fpas_parser::parse(
        "program T; uses Std.Console; begin Std.Console.WriteLn('abc'.Length()); end.",
    );
    assert!(parse_errors.is_empty(), "{parse_errors:#?}");
    let metadata = analyze_with_types(&program);
    assert!(metadata.errors.is_empty(), "{:?}", metadata.errors);
    let calls = metadata.intrinsic_calls.values().collect::<Vec<_>>();
    assert!(
        metadata
            .fluent_calls
            .values()
            .any(|call| call.name == "Std.Str.Length"),
        "{:?}",
        metadata.fluent_calls
    );
    assert!(
        calls
            .iter()
            .any(|call| call.as_str() == "Std.Console.WriteLn"),
        "{calls:?}"
    );
}

#[test]
fn analyze_with_types_canonicalizes_short_standard_intrinsic_calls() {
    let (program, parse_errors) =
        fpas_parser::parse("program T; uses Std.Console; begin WriteLn('hello'); end.");
    assert!(parse_errors.is_empty(), "{parse_errors:#?}");
    let metadata = analyze_with_types(&program);
    assert!(metadata.errors.is_empty(), "{:?}", metadata.errors);
    assert_eq!(
        metadata.intrinsic_calls.values().collect::<Vec<_>>(),
        vec!["Std.Console.WriteLn"]
    );
}

#[test]
fn analysis_metadata_exposes_all_named_results() {
    let (program, parse_errors) = fpas_parser::parse("program T; begin end.");
    assert!(parse_errors.is_empty(), "{parse_errors:#?}");

    let crate::AnalysisMetadata {
        errors,
        import_aliases,
        expr_types,
        intrinsic_calls,
        named_argument_orders,
        named_types,
        method_calls,
        fluent_calls,
        member_value_calls,
        record_defaults,
        record_constructions,
        scalar_case_bindings,
        closure_infos,
        nested_routine_captures,
        bound_methods,
        event_writes,
        event_assigned,
        event_raises,
    } = analyze_with_types(&program);

    assert_eq!(named_types.len(), 4);
    assert!(record_constructions.is_empty());
    assert_eq!(
        [
            errors.len(),
            import_aliases.len(),
            expr_types.len(),
            intrinsic_calls.len(),
            named_argument_orders.len(),
            method_calls.len(),
            fluent_calls.len(),
            member_value_calls.len(),
            record_defaults.len(),
            scalar_case_bindings.len(),
            closure_infos.len(),
            nested_routine_captures.len(),
            bound_methods.len(),
            event_writes.len(),
            event_assigned.len(),
            event_raises.len(),
        ],
        [0; 16]
    );
}

#[test]
fn equality_records_with_comparable_fields_are_valid() {
    check_ok(
        "program T; type Id = record Value: integer; end record; const A: Id := Id( Value := 1 ); const B: Id := Id( Value := 1 ); const Same: boolean := A = B; begin end.",
    );
}

// ── Shift ───────────────────────────────────────────────────────

#[test]
fn shift_left_valid() {
    check_ok("program T; uses Std.Bits; const X: integer := ShiftLeft(1, 4); begin end.");
}

#[test]
fn shift_right_valid() {
    check_ok("program T; uses Std.Bits; const X: integer := ShiftRight(16, 4); begin end.");
}

#[test]
fn shift_left_with_real_error() {
    check_errors("program T; uses Std.Bits; const X: integer := ShiftLeft(1, 2.0); begin end.");
}

#[test]
fn shift_right_with_real_error() {
    check_errors("program T; uses Std.Bits; const X: integer := ShiftRight(16, 1.5); begin end.");
}

#[test]
fn mod_with_real_error() {
    check_errors("program T; const X: integer := 10 mod 3.0; begin end.");
}

#[test]
fn xor_booleans() {
    check_ok("program T; const X: boolean := true xor false; begin end.");
}

#[test]
fn bit_xor_integers() {
    check_ok("program T; uses Std.Bits; const X: integer := BitXor(5, 3); begin end.");
}

#[test]
fn xor_with_string_error() {
    check_errors("program T; const X: boolean := 'a' xor 'b'; begin end.");
}

#[test]
fn and_with_string_error() {
    check_errors("program T; const X: boolean := 'a' and 'b'; begin end.");
}

#[test]
fn or_with_string_error() {
    check_errors("program T; const X: boolean := 'a' or 'b'; begin end.");
}

#[test]
fn not_string_error() {
    check_errors("program T; const X: boolean := not 'hello'; begin end.");
}

#[test]
fn bit_not_integer() {
    check_ok("program T; uses Std.Bits; const X: integer := BitNot(0); begin end.");
}

#[test]
fn compare_incompatible_types_error() {
    check_errors("program T; const X: boolean := 1 < 'hello'; begin end.");
}

#[test]
fn negate_real() {
    check_ok("program T; const X: real := -3.14; begin end.");
}

#[test]
fn not_real_error() {
    check_errors("program T; const X: real := not 3.14; begin end.");
}

// ── Array literal ───────────────────────────────────────────────

#[test]
fn array_literal_valid() {
    check_ok("program T; const X: array of integer := [1, 2, 3]; begin end.");
}

#[test]
fn array_literal_mixed_types() {
    check_errors("program T; const X: array of integer := [1, 2, true]; begin end.");
}

#[test]
fn empty_array() {
    check_ok("program T; const X: array of integer := []; begin end.");
}

// ── Designator ──────────────────────────────────────────────────

#[test]
fn undefined_variable() {
    check_errors("program T; begin return Foo; end.");
}

// ── Function call ───────────────────────────────────────────────

#[test]
fn call_function() {
    check_ok(
        "program T; \
         function Add(A: integer; B: integer): integer; \
         begin return A + B; end function; \
         begin const X: integer := Add(1, 2); end.",
    );
}

#[test]
fn call_wrong_arg_count() {
    check_errors(
        "program T; \
         function Add(A: integer; B: integer): integer; \
         begin return A + B; end function; \
         begin const X: integer := Add(1); end.",
    );
}
