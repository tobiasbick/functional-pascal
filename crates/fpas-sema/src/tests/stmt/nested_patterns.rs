//! Recursive pattern typing, coverage, bindings, and ordered alternatives.

use super::super::{check_errors, check_ok};
use fpas_diagnostics::codes::{
    SEMA_DUPLICATE_DECLARATION, SEMA_NON_EXHAUSTIVE_CASE, SEMA_TYPE_MISMATCH,
};

#[test]
fn nested_patterns_cover_payload_products_and_resolve_generic_bindings() {
    check_ok(
        r#"program Main;
        type Choice of (T) = enum Present(Value: T); Missing; end enum;
        type Pair = enum Both(A: boolean; B: boolean); Empty; end enum;
        const Item: Choice of (Option of (integer)) := Choice.Present(Option.Some(42));
        const Flags: Pair := Pair.Both(true, false);
        begin
          case Item of
            when Choice.Present(Option.Some(const Value)): const Next: integer := Value + 1;
            when Choice.Present(Option.None): null;
            when Choice.Missing: null;
          end case;
          case Flags of
            when Pair.Both(true, _): null;
            when Pair.Both(false, true): null;
            when Pair.Both(false, false): null;
            when Pair.Empty: null;
          end case;
        end program;"#,
    );
}

#[test]
fn nested_pattern_coverage_requires_payload_alternatives_and_ignores_guards() {
    for arms in [
        "when Choice.Present(true): null; when Choice.Missing: null;",
        "when Choice.Present(_): null; when Choice.Missing if true: null;",
    ] {
        let errors = check_errors(&format!(
            r#"program Main;
            type Choice = enum Present(Value: boolean); Missing; end enum;
            const Item: Choice := Choice.Missing;
            begin case Item of {arms} end case; end program;"#
        ));
        assert!(
            errors
                .iter()
                .any(|error| error.code == SEMA_NON_EXHAUSTIVE_CASE),
            "{errors:#?}"
        );
    }
}

#[test]
fn nested_patterns_reject_duplicate_bindings_and_import_qualifier_shadowing() {
    for (imports, pattern) in [
        ("", "Pair.Both(const Value, const value)"),
        (
            "uses Std.Console as Console;",
            "Pair.Both(const Console, _)",
        ),
    ] {
        let errors = check_errors(&format!(
            r#"program Main; {imports}
            type Pair = enum Both(A: integer; B: integer); end enum;
            begin const Item: Pair := Pair.Both(1, 2);
            case Item of when {pattern}: null; end case; end program;"#
        ));
        assert!(
            errors
                .iter()
                .any(|error| error.code == SEMA_DUPLICATE_DECLARATION),
            "{errors:#?}"
        );
    }
}

#[test]
fn nested_patterns_reject_non_constants_wrong_owners_and_grouped_binding_mismatches() {
    for patterns in [
        "when Choice.Present(Input): null; when Choice.Present(_): null; when Choice.Missing: null;",
        "when Choice.Present(const Value), Choice.Missing: null; when Choice.Present(_): null;",
        "when Choice.Present(Other.Present(_)): null; when Choice.Present(_): null; when Choice.Missing: null;",
        "when Choice.Present(_): null; else null;",
    ] {
        let errors = check_errors(&format!(
            r#"program Main;
            type Choice = enum Present(Value: integer); Missing; end enum;
            type Other = enum Present(Value: integer); end enum;
            var Input: integer := 1; const Item: Choice := Choice.Present(1);
            begin case Item of {patterns} end case; end program;"#
        ));
        assert!(!errors.is_empty(), "{patterns}");
    }
}

#[test]
fn nested_coverage_rejects_unreachable_rows_and_accepts_guarded_fallbacks() {
    let source = |arms: &str| {
        format!(
            r#"program Main;
        type Choice = enum Present(Value: integer); Missing; end enum;
        const Item: Choice := Choice.Present(42);
        begin case Item of {arms} end case; end program;"#
        )
    };
    let errors = check_errors(&source(
        "when Choice.Present(_): null; when Choice.Present(42): null; when Choice.Missing: null;",
    ));
    assert!(
        errors
            .iter()
            .any(|error| error.code == SEMA_TYPE_MISMATCH && error.message.contains("unreachable")),
        "{errors:#?}"
    );
    check_ok(&source(
        "when Choice.Present(const Value) if Value > 0: null; when Choice.Present(_): null; when Choice.Missing: null;",
    ));
}

#[test]
fn static_boolean_payload_constants_establish_finite_coverage() {
    check_ok(
        r#"program Main;
        const Enabled: boolean := not false;
        type Choice = enum Present(Value: boolean); end enum;
        const Item: Choice := Choice.Present(true);
        begin case Item of
            when Choice.Present(Enabled): null;
            when Choice.Present(false): null;
        end case; end program;"#,
    );
}

#[test]
fn nested_scalar_ranges_detect_union_coverage_and_do_not_cover_other_payloads() {
    for (ty, initializer, ranges, duplicate) in [
        (
            "integer",
            "42",
            "when Choice.Present(1..10): null; when Choice.Present(11..20): null;",
            "when Choice.Present(5..15): null;",
        ),
        (
            "string",
            "'m'",
            "when Choice.Present('a'..'z'): null;",
            "when Choice.Present('m'): null;",
        ),
    ] {
        let errors = check_errors(&format!(
            r#"program Main; type Choice = enum Present(Value: {ty}); end enum;
            const Item: Choice := Choice.Present({initializer});
            begin case Item of {ranges} {duplicate} when Choice.Present(_): null; end case; end program;"#
        ));
        assert!(
            errors
                .iter()
                .any(|error| error.message.contains("unreachable")),
            "{errors:#?}"
        );
    }
    check_ok(
        r#"program Main; type Pair = enum Both(A: integer; B: boolean); end enum;
        const Item: Pair := Pair.Both(1, true);
        begin case Item of
          when Pair.Both(1..10, true): null;
          when Pair.Both(1..10, false): null;
          when Pair.Both(_, _): null;
        end case; end program;"#,
    );
}

#[test]
fn explicit_pattern_bindings_shadow_outer_locals_and_remain_immutable_and_arm_local() {
    check_ok(
        r#"program Main; type Choice = enum Present(Value: integer); Missing; end enum;
        const Value: string := 'outer'; const Item: Choice := Choice.Present(42);
        begin case Item of
          when Choice.Present(const Value) if Value = 42: null;
          when Choice.Present(_): null;
          when Choice.Missing: null;
        end case; const Text: string := Value; end program;"#,
    );
    for body in ["Value := 1;", "var Value: integer := 1;"] {
        let errors = check_errors(&format!(
            r#"program Main; type Choice = enum Present(Value: integer); end enum;
            const Item: Choice := Choice.Present(42);
            begin case Item of when Choice.Present(const Value): {body} end case; end program;"#
        ));
        assert!(!errors.is_empty(), "{body}");
    }
    let errors = check_errors(
        r#"program Main; type Choice = enum Present(Value: integer); end enum;
        const Item: Choice := Choice.Present(42);
        begin case Item of when Choice.Present(const Value): null; end case;
        const After: integer := Value; end program;"#,
    );
    assert!(
        errors
            .iter()
            .any(|error| error.code == fpas_diagnostics::codes::SEMA_UNKNOWN_NAME),
        "{errors:#?}"
    );
}

#[test]
fn canonical_patterns_reject_payloadless_calls_and_top_level_catchalls() {
    for labels in [
        "when Choice.Missing(): null; when Choice.Present(_): null;",
        "when _: null;",
        "when const Whole: null;",
        "when Present(_): null;",
    ] {
        let errors = check_errors(&format!(
            r#"program Main; type Choice = enum Present(Value: integer); Missing; end enum;
            const Item: Choice := Choice.Missing;
            begin case Item of {labels} end case; end program;"#
        ));
        assert!(
            errors.iter().any(|error| error.code == SEMA_TYPE_MISMATCH),
            "{labels}: {errors:#?}"
        );
    }
}
