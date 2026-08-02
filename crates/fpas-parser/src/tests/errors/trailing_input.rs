use super::*;
use crate::Stmt;
use fpas_diagnostics::codes::PARSE_EXPECTED_TOKEN;

#[test]
fn identifier_after_program_terminator_is_rejected_without_expanding_program_span() {
    let valid_source = "program T; begin end.";
    let (program, errors) = parse_with_errors(&format!("{valid_source} Garbage"));
    let parser_errors: Vec<_> = errors
        .iter()
        .filter_map(ParseDiagnostic::as_parser_error)
        .collect();

    assert_eq!(program.span.length, valid_source.len());
    assert_eq!(parser_errors.len(), 1, "diagnostics: {parser_errors:#?}");
    assert_eq!(parser_errors[0].code, PARSE_EXPECTED_TOKEN);
    assert!(
        parser_errors[0]
            .message
            .contains("after program terminator")
    );
}

#[test]
fn second_program_after_terminator_is_rejected_once() {
    let (program, errors) =
        parse_with_errors("program First; begin end. program Second; begin end.");
    let parser_errors: Vec<_> = errors
        .iter()
        .filter_map(ParseDiagnostic::as_parser_error)
        .collect();

    assert_eq!(program.name, "First");
    assert_eq!(parser_errors.len(), 1, "diagnostics: {parser_errors:#?}");
    assert!(parser_errors[0].message.contains("found `program`"));
}

#[test]
fn literal_after_program_terminator_is_rejected() {
    let (program, errors) = parse_with_errors("program T; begin X := 1 end. 42");
    let parser_errors: Vec<_> = errors
        .iter()
        .filter_map(ParseDiagnostic::as_parser_error)
        .collect();

    assert!(matches!(program.body.as_slice(), [Stmt::Assign { .. }]));
    assert_eq!(parser_errors.len(), 1, "diagnostics: {parser_errors:#?}");
    assert!(parser_errors[0].message.contains("found `42`"));
}
