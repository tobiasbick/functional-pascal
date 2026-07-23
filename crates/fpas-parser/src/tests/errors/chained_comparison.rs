use super::parse_with_errors;
use crate::ParseDiagnostic;
use fpas_diagnostics::codes::PARSE_EXPECTED_EXPRESSION;

#[test]
fn chained_comparison_reports_dedicated_error() {
    let (_, errors) = parse_with_errors("program T; begin X := 1 = 2 = 3 end.");
    let parser_errors: Vec<_> = errors
        .iter()
        .filter_map(ParseDiagnostic::as_parser_error)
        .collect();
    assert_eq!(
        parser_errors.len(),
        1,
        "unexpected parser diagnostics: {parser_errors:#?}"
    );
    assert_eq!(parser_errors[0].code, PARSE_EXPECTED_EXPRESSION);
    assert!(
        parser_errors[0]
            .message
            .contains("Chained comparison operators are not allowed")
    );
}

#[test]
fn longer_chained_comparison_recovers_all_extra_operators() {
    let (program, errors) = parse_with_errors("program T; begin X := 1 = 2 = 3 = 4; Y := 5 end.");
    let parser_errors: Vec<_> = errors
        .iter()
        .filter_map(ParseDiagnostic::as_parser_error)
        .collect();
    assert_eq!(
        parser_errors.len(),
        1,
        "unexpected parser diagnostics: {parser_errors:#?}"
    );
    assert!(
        parser_errors[0]
            .message
            .contains("Chained comparison operators are not allowed")
    );
    assert_eq!(program.body.len(), 2);
    assert!(matches!(program.body[1], crate::Stmt::Assign { .. }));
}
