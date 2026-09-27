use super::parse_with_errors;
use crate::ParseDiagnostic;
use fpas_diagnostics::codes::{PARSE_EXPECTED_EXPRESSION, PARSE_EXPECTED_TOKEN};

#[test]
fn valid_expression_delimiters_parse_without_diagnostics() {
    for expression in [
        "F(1, 2)",
        "[1, 2]",
        "['a': 1, 'b': 2]",
        "record X := 1; end",
        "Some([:])",
    ] {
        let source = format!("program T; begin return {expression} end.");
        let (_, errors) = parse_with_errors(&source);
        assert!(errors.is_empty(), "{expression}: {errors:#?}");
    }
}

#[test]
fn trailing_delimiters_and_missing_record_separator_are_rejected() {
    for (expression, expected_code) in [
        ("F(1,)", PARSE_EXPECTED_EXPRESSION),
        ("[1,]", PARSE_EXPECTED_EXPRESSION),
        ("['a': 1,]", PARSE_EXPECTED_EXPRESSION),
        ("record X := 1 end", PARSE_EXPECTED_TOKEN),
        ("Some()", PARSE_EXPECTED_EXPRESSION),
    ] {
        let source = format!("program T; begin return {expression} end.");
        let (_, errors) = parse_with_errors(&source);
        assert!(
            errors.iter().any(|error| matches!(error, ParseDiagnostic::Parser(diagnostic) if diagnostic.code == expected_code)),
            "{expression}: {errors:#?}"
        );
    }
}
