use crate::{Expr, ParseDiagnostic, parse_expression};

#[test]
fn standalone_expression_preserves_precedence() {
    let (expression, diagnostics) = parse_expression("1 + 2 * 3");

    assert!(
        diagnostics.is_empty(),
        "unexpected diagnostics: {diagnostics:#?}"
    );
    assert!(matches!(expression, Expr::BinaryOp { .. }));
}

#[test]
fn standalone_expression_rejects_trailing_tokens() {
    let (_, diagnostics) = parse_expression("Counter extra");

    assert!(matches!(
        diagnostics.as_slice(),
        [ParseDiagnostic::Parser(_)]
    ));
    assert!(
        diagnostics[0]
            .as_diagnostic()
            .message
            .contains("end of expression")
    );
}

#[test]
fn standalone_expression_orders_lexer_diagnostics_first() {
    let (_, diagnostics) = parse_expression("@ extra");

    assert!(matches!(
        diagnostics.first(),
        Some(ParseDiagnostic::Lexer(_))
    ));
}

#[test]
fn unsigned_minimum_magnitude_requires_direct_negative_sign() {
    use fpas_diagnostics::codes::PARSE_INTEGER_LITERAL_OVERFLOW;

    for source in [
        "9223372036854775808",
        "$8000000000000000",
        "-(9223372036854775808)",
        "-($8000000000000000)",
    ] {
        let (_, diagnostics) = parse_expression(source);
        assert_eq!(diagnostics.len(), 1, "{source}: {diagnostics:?}");
        assert_eq!(
            diagnostics[0].as_diagnostic().code,
            PARSE_INTEGER_LITERAL_OVERFLOW,
            "{source}"
        );
        assert!(
            diagnostics[0]
                .as_diagnostic()
                .help
                .as_ref()
                .is_some_and(|help| help.contains("-9223372036854775808"))
        );
    }
}
