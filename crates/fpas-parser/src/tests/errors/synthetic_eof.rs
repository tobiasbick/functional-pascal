use crate::{ParseDiagnostic, parse_tokens_compilation_unit};
use fpas_lexer::{Token, lex_with_source_id};

#[test]
fn synthetic_eof_matches_lexer_position_after_unicode_and_line_endings() {
    for source in [
        "program T; begin 'é'",
        "program T; begin 'a\nb'",
        "program T; begin 'a\r\nb'",
        "program T; begin 'a\rb'",
    ] {
        let (mut tokens, _, lex_errors) = lex_with_source_id(source, 17);
        assert!(
            lex_errors.is_empty(),
            "source: {source:?}; errors: {lex_errors:#?}"
        );
        let real_eof = tokens.pop().expect("lexer must append EOF");
        assert!(matches!(real_eof.token, Token::Eof));

        let (_, errors) = parse_tokens_compilation_unit(tokens);
        let synthetic_eof_error = errors
            .iter()
            .filter_map(ParseDiagnostic::as_parser_error)
            .next_back()
            .expect("truncated program must report its missing terminator");

        assert_eq!(
            synthetic_eof_error.span,
            real_eof.span.diagnostic_span_or_synthetic(),
            "source: {source:?}"
        );
    }
}
