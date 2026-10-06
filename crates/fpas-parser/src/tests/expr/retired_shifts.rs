//! Removed infix shifts have migration diagnostics while their names remain identifiers.

use crate::{Expr, Stmt, parse, parse_expression};
use fpas_diagnostics::codes::PARSE_EXPECTED_EXPRESSION;

#[test]
fn infix_shifts_report_the_matching_std_bits_function() {
    for (operator, function) in [
        ("shl", "ShiftLeft"),
        ("shr", "ShiftRight"),
        ("ShL", "ShiftLeft"),
        ("SHR", "ShiftRight"),
    ] {
        for source in [
            format!("1 {operator} 2"),
            format!("(A + B) {operator} Count"),
            format!("F(1 {operator} 2)"),
        ] {
            let (_, errors) = parse_expression(&source);
            let diagnostic = errors
                .iter()
                .map(|error| error.as_diagnostic())
                .find(|error| error.message.contains("is not an operator"))
                .expect("shift migration diagnostic");
            assert_eq!(diagnostic.code, PARSE_EXPECTED_EXPRESSION);
            let hint = diagnostic.help.as_deref().expect("migration hint");
            assert!(hint.contains(&format!("Std.Bits.{function}")), "{hint}");
            assert!(hint.contains("uses Std.Bits;"));
            let span = diagnostic.span.expect("operator span");
            assert_eq!(
                &source[span.offset()..span.offset() + span.length()],
                operator
            );
        }
    }
}

#[test]
fn retired_shift_names_are_valid_operands_calls_and_members() {
    for source in [
        "shl",
        "SHR",
        "shl(1)",
        "Shr(2)",
        "Value.shl",
        "Value.shr(1)",
        "shl + shr",
        "shl(shr(1))",
    ] {
        assert!(parse_expression(source).1.is_empty(), "{source}");
    }
}

#[test]
fn shift_recovery_preserves_the_following_statement() {
    for source in ["1 shl 2 shr 3", "1 shr", "1 shl (2 + 3)"] {
        let (program, errors) = parse(&format!(
            "program T; begin var Value: integer := {source}; return 42; end."
        ));
        assert!(!errors.is_empty(), "{source}");
        assert!(
            matches!(
                program.body.last(),
                Some(Stmt::Return(Some(Expr::Integer(42, _)), _))
            ),
            "{source}: {program:?}"
        );
    }
}
