use super::toks;
use crate::Token;

#[test]
fn simple_identifiers() {
    assert_eq!(toks("Foo"), vec![Token::Ident("Foo".into())]);
    assert_eq!(toks("bar"), vec![Token::Ident("bar".into())]);
    assert_eq!(toks("MyVar"), vec![Token::Ident("MyVar".into())]);
}

#[test]
fn underscore_start() {
    assert_eq!(toks("_foo"), vec![Token::Ident("_foo".into())]);
    assert_eq!(toks("_"), vec![Token::Ident("_".into())]);
    assert_eq!(toks("__"), vec![Token::Ident("__".into())]);
    assert_eq!(toks("___x"), vec![Token::Ident("___x".into())]);
}

#[test]
fn with_digits() {
    assert_eq!(toks("x1"), vec![Token::Ident("x1".into())]);
    assert_eq!(toks("point2d"), vec![Token::Ident("point2d".into())]);
    assert_eq!(toks("abc123"), vec![Token::Ident("abc123".into())]);
    assert_eq!(toks("_123"), vec![Token::Ident("_123".into())]);
}

#[test]
fn all_caps() {
    assert_eq!(toks("MAX_SIZE"), vec![Token::Ident("MAX_SIZE".into())]);
    assert_eq!(
        toks("MY_CONST_42"),
        vec![Token::Ident("MY_CONST_42".into())]
    );
}

#[test]
fn single_character() {
    assert_eq!(toks("x"), vec![Token::Ident("x".into())]);
    assert_eq!(toks("X"), vec![Token::Ident("X".into())]);
}

#[test]
fn adjacent_identifiers() {
    assert_eq!(
        toks("foo bar"),
        vec![Token::Ident("foo".into()), Token::Ident("bar".into())]
    );
}

#[test]
fn identifier_preserves_case() {
    assert_eq!(toks("MyVariable"), vec![Token::Ident("MyVariable".into())]);
    assert_eq!(toks("ALLCAPS"), vec![Token::Ident("ALLCAPS".into())]);
}

#[test]
fn long_identifier() {
    let name = "VeryLongIdentifierNameForTestingPurposes";
    assert_eq!(toks(name), vec![Token::Ident(name.into())]);
}

#[test]
fn non_ascii_letter_in_identifier_is_rejected() {
    use super::lex_with_errors;
    use fpas_diagnostics::codes::LEX_NON_ASCII_IN_IDENTIFIER;

    let (toks, errs) = lex_with_errors("caf\u{e9}");
    assert!(
        toks.is_empty(),
        "partial ASCII prefix must not become an Ident"
    );
    assert_eq!(errs.len(), 1);
    assert_eq!(errs[0].code, LEX_NON_ASCII_IN_IDENTIFIER);
}

#[test]
fn invalid_identifier_recovery_consumes_ascii_remainder() {
    use super::lex_with_errors;
    use fpas_diagnostics::codes::LEX_NON_ASCII_IN_IDENTIFIER;

    let source = "fooébar + next";
    let (tokens, errors) = lex_with_errors(source);
    assert_eq!(tokens, vec![Token::Plus, Token::Ident("next".to_string())]);
    assert_eq!(errors.len(), 1);
    assert_eq!(errors[0].code, LEX_NON_ASCII_IN_IDENTIFIER);
    assert_eq!(errors[0].span.offset(), 0);
    assert_eq!(errors[0].span.length(), "fooébar".len());
}

#[test]
fn invalid_identifier_recovery_consumes_multiple_mixed_runs() {
    use super::lex_with_errors;

    let (tokens, errors) = lex_with_errors("fooébarβ_baz qux");
    assert_eq!(tokens, vec![Token::Ident("qux".to_string())]);
    assert_eq!(errors.len(), 1);
}

#[test]
fn non_ascii_initial_identifier_is_recovered_as_one_unit() {
    use super::lex_with_errors;
    use fpas_diagnostics::codes::LEX_NON_ASCII_IN_IDENTIFIER;

    let (tokens, errors) = lex_with_errors("ébar; next");
    assert_eq!(
        tokens,
        vec![Token::Semicolon, Token::Ident("next".to_string())]
    );
    assert_eq!(errors.len(), 1);
    assert_eq!(errors[0].code, LEX_NON_ASCII_IN_IDENTIFIER);
}
