//! Boolean-only logical typing, alias resolution, and integer migration diagnostics.

use super::{check_errors, check_ok};
use fpas_diagnostics::codes::SEMA_TYPE_MISMATCH;

#[test]
fn integer_logical_operators_suggest_the_matching_bits_function() {
    for (expression, function) in [
        ("1 and 2", "BitAnd"),
        ("1 or 2", "BitOr"),
        ("1 xor 2", "BitXor"),
        ("not 1", "BitNot"),
    ] {
        let errors = check_errors(&format!(
            "program T; begin const Value: integer := {expression}; end."
        ));
        assert_eq!(errors.len(), 1, "{expression}: {errors:?}");
        assert_eq!(errors[0].code, SEMA_TYPE_MISMATCH);
        assert!(errors[0].message.contains("boolean"));
        let hint = errors[0].help.as_deref().expect("migration hint");
        assert!(hint.contains(&format!("Std.Bits.{function}")), "{hint}");
        assert!(hint.contains("uses Std.Bits;"));
    }
}

#[test]
fn mixed_and_other_non_boolean_operands_are_rejected_without_integer_hints() {
    for operator in ["and", "or", "xor"] {
        for (left, right) in [
            ("true", "1"),
            ("0", "false"),
            ("false", "1.0"),
            ("'text'", "true"),
            ("[1]", "false"),
        ] {
            let expression = format!("{left} {operator} {right}");
            let errors = check_errors(&format!(
                "program T; begin const Value: boolean := {expression}; end."
            ));
            assert_eq!(errors.len(), 1, "{expression}: {errors:?}");
            assert_eq!(errors[0].code, SEMA_TYPE_MISMATCH);
            let hint = errors[0].help.as_deref().expect("Boolean hint");
            assert!(hint.contains("Both operands must be boolean"), "{hint}");
            assert!(!hint.contains("Std.Bits"), "{hint}");
        }
    }
}

#[test]
fn aliases_keep_boolean_and_integer_operator_rules() {
    check_ok(
        "program T; type Flag = boolean; const A: Flag := true; const B: Flag := false; begin const C: boolean := (A and B) xor not A; end.",
    );
    let errors = check_errors(
        "program T; type Bits = integer; const A: Bits := 1; begin const B: Bits := A or A; end.",
    );
    assert!(
        errors[0]
            .help
            .as_deref()
            .expect("hint")
            .contains("Std.Bits.BitOr")
    );
}

#[test]
fn invalid_names_do_not_cascade_into_logical_type_errors() {
    for expression in [
        "not Missing",
        "Missing and true",
        "false or Missing",
        "Missing xor false",
    ] {
        let errors = check_errors(&format!(
            "program T; begin const Value: boolean := {expression}; end."
        ));
        assert_eq!(errors.len(), 1, "{expression}: {errors:?}");
        assert_ne!(errors[0].code, SEMA_TYPE_MISMATCH);
    }
}
