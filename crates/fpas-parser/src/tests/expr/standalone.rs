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
