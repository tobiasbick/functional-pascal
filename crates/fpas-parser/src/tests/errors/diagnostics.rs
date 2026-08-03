use super::*;

#[test]
fn errors_have_hints() {
    let (_, errs) = parse_with_errors("program T; begin @ end.");
    for err in &errs {
        match err {
            ParseDiagnostic::Lexer(diagnostic) => {
                assert!(
                    diagnostic
                        .help
                        .as_deref()
                        .map(|hint| !hint.is_empty())
                        .unwrap_or(false),
                    "Error missing hint: {}",
                    diagnostic.message
                );
            }
            ParseDiagnostic::Parser(error) => assert!(
                error.help.as_deref().is_some_and(|hint| !hint.is_empty()),
                "Error missing hint: {}",
                error.message
            ),
        }
    }
}

#[test]
fn expected_token_has_correct_code() {
    use fpas_diagnostics::codes::PARSE_EXPECTED_TOKEN;

    let (_, errs) = parse_with_errors("program Hello begin end.");
    assert!(!errs.is_empty());
    let parse_err = errs.iter().find_map(ParseDiagnostic::as_parser_error);
    let d = parse_err.expect("expected a parser diagnostic");
    assert_eq!(d.code, PARSE_EXPECTED_TOKEN, "wrong diagnostic code");
    assert_eq!(d.span.line(), 1, "wrong line");
    assert!(
        d.help.as_deref().is_some_and(|h| !h.is_empty()),
        "help text must be present"
    );
}

#[test]
fn expected_expression_has_correct_code() {
    use fpas_diagnostics::codes::PARSE_EXPECTED_EXPRESSION;

    let (_, errs) = parse_with_errors("program T; begin X := end.");
    let parse_err = errs.iter().find_map(|d| {
        d.as_parser_error()
            .filter(|e| e.code == PARSE_EXPECTED_EXPRESSION)
    });
    assert!(
        parse_err.is_some(),
        "expected PARSE_EXPECTED_EXPRESSION; got: {errs:#?}"
    );
    let d = parse_err.unwrap();
    assert!(
        d.help.as_deref().is_some_and(|h| !h.is_empty()),
        "help text must be present"
    );
}
