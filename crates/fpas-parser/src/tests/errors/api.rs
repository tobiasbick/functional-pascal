use super::{parse_compilation_unit_with_errors, parse_with_errors};
use crate::{CompilationUnit, ParseDiagnostic, parse_tokens_compilation_unit};
use fpas_diagnostics::codes::PARSE_EXPECTED_TOKEN;
use fpas_lexer::{lex, lex_with_source_id};

#[test]
fn parse_tokens_compilation_unit_accepts_empty_stream_without_panicking() {
    let (unit, errors) = parse_tokens_compilation_unit(vec![]);
    assert_eq!(errors.len(), 1);
    assert_eq!(errors[0].as_diagnostic().code, PARSE_EXPECTED_TOKEN);
    assert!(matches!(unit, CompilationUnit::Program(_)));
}

#[test]
fn parse_tokens_compilation_unit_parses_lexed_unit() {
    let source = "unit Demo; function Answer(): integer; begin return 42 end;";
    let (tokens, _, lex_errors) = lex_with_source_id(source, 7);
    assert!(lex_errors.is_empty());

    let (unit, parse_errors) = parse_tokens_compilation_unit(tokens);
    assert!(parse_errors.is_empty());

    let CompilationUnit::Unit(unit) = unit else {
        panic!("expected unit compilation unit");
    };
    assert_eq!(unit.name.parts, vec!["Demo"]);
    assert_eq!(unit.declarations.len(), 1);
}

#[test]
fn parse_tokens_compilation_unit_matches_source_api() {
    let source = "program T; begin end.";
    let (tokens, _) = lex(source);
    let (unit_from_tokens, token_errors) = parse_tokens_compilation_unit(tokens);
    let (unit_from_source, source_errors) = parse_compilation_unit_with_errors(source);

    assert!(token_errors.is_empty());
    assert!(source_errors.is_empty());
    assert_eq!(unit_from_tokens, unit_from_source);
}

#[test]
fn wrong_compilation_unit_header_emits_single_parser_error() {
    let (_, errors) = parse_compilation_unit_with_errors("const X = 1;");
    let parser_errors: Vec<_> = errors
        .iter()
        .filter_map(ParseDiagnostic::as_parser_error)
        .collect();
    assert_eq!(
        parser_errors.len(),
        1,
        "unexpected parser diagnostics: {parser_errors:#?}"
    );
    assert_eq!(parser_errors[0].code, PARSE_EXPECTED_TOKEN);
    assert!(
        parser_errors[0]
            .message
            .contains("Expected `program` or `unit`")
    );
}

#[test]
fn wrong_program_header_emits_single_parser_error() {
    let (_, errors) = parse_with_errors("const X = 1;");
    let parser_errors: Vec<_> = errors
        .iter()
        .filter_map(ParseDiagnostic::as_parser_error)
        .collect();
    assert_eq!(
        parser_errors.len(),
        1,
        "unexpected parser diagnostics: {parser_errors:#?}"
    );
    assert_eq!(parser_errors[0].code, PARSE_EXPECTED_TOKEN);
    assert!(
        parser_errors[0].message.contains("Expected `program`"),
        "message: {}",
        parser_errors[0].message
    );
}

#[test]
fn source_api_orders_lexer_diagnostics_before_parser_diagnostics() {
    let (_, errors) = parse_with_errors("program T begin @ end.");

    assert!(matches!(
        errors.first(),
        Some(crate::ParseDiagnostic::Lexer(_))
    ));
    assert!(
        errors
            .iter()
            .skip(1)
            .any(|error| matches!(error, crate::ParseDiagnostic::Parser(_))),
        "diagnostics: {errors:#?}"
    );
}
