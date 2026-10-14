use super::super::lex_with_errors;
use crate::Token;
use fpas_diagnostics::codes::{LEX_INVALID_CHARACTER_CODE_LITERAL, LEX_INVALID_DIGIT_SEPARATOR};

#[test]
fn unterminated_string() {
    let (toks, errs) = lex_with_errors("'not closed");
    assert_eq!(errs.len(), 1);
    assert!(errs[0].message.contains("Unterminated string"));
    assert_eq!(toks.len(), 1);
}

#[test]
fn unterminated_string_with_escape() {
    let (_, errs) = lex_with_errors("'it''s not closed");
    assert_eq!(errs.len(), 1);
}

#[test]
fn hash_alone() {
    let (toks, errs) = lex_with_errors("#");
    assert_eq!(errs.len(), 1);
    assert!(errs[0].message.contains("#"));
    assert_eq!(toks, vec![Token::Str("".into())]);
}

#[test]
fn hash_non_digit() {
    let (toks, errs) = lex_with_errors("#abc");
    assert_eq!(errs.len(), 1);
    assert_eq!(
        toks,
        vec![Token::Str("".into()), Token::Ident("abc".into())]
    );
}

#[test]
fn invalid_character_codes_have_precise_spans_and_recover_without_forging_strings() {
    let oversized = format!("#{}", "9".repeat(128));
    for code in [
        "#55296",
        "#57343",
        "#1114112",
        "#4294967295",
        "#4294967296",
        &oversized,
    ] {
        let source = format!(" \r\n'😀'{code}'B';");
        let (tokens, _, errors) = crate::lex_with_source_id(&source, 23);
        assert_eq!(errors.len(), 1, "{code}: {errors:?}");
        let error = &errors[0];
        assert_eq!(error.code, LEX_INVALID_CHARACTER_CODE_LITERAL, "{code}");
        let span = error.span.expect("invalid character-code span");
        assert_eq!(span.source_id(), 23);
        assert_eq!(span.offset(), 9);
        assert_eq!(span.length(), code.len());
        assert_eq!((span.line(), span.column()), (2, 4));
        let hint = error.help.as_deref().expect("scalar repair hint");
        assert!(hint.contains("0..1114111"), "{hint}");
        assert!(hint.contains("55296..57343"), "{hint}");
        assert!(hint.contains("#8364"), "{hint}");
        assert_eq!(tokens[0].token, Token::Str("😀".into()));
        assert_eq!(tokens[1].token, Token::Str("B".into()));
        assert_eq!(tokens[2].token, Token::Semicolon);
        assert_eq!(tokens[3].token, Token::Eof);
    }
}

#[test]
fn character_codes_retain_digit_separator_rejection() {
    for source in ["#8__364", "#1__114__111"] {
        let (_, errors) = lex_with_errors(source);
        assert!(
            errors
                .iter()
                .all(|error| error.code == LEX_INVALID_DIGIT_SEPARATOR),
            "{source}: {errors:?}"
        );
        assert!(!errors.is_empty(), "{source}");
    }
}

#[test]
fn char_code_error_has_explicit_help() {
    let (_, errs) = lex_with_errors("#");
    assert_eq!(errs.len(), 1);
    assert!(
        errs[0]
            .help
            .as_deref()
            .is_some_and(|hint| hint.contains("0..1114111") && hint.contains("55296..57343"))
    );
}

#[test]
fn unterminated_string_has_correct_code() {
    use fpas_diagnostics::codes::LEX_UNTERMINATED_STRING_LITERAL;

    let (_, errs) = lex_with_errors("'unclosed");
    assert_eq!(errs.len(), 1);
    assert_eq!(
        errs[0].code, LEX_UNTERMINATED_STRING_LITERAL,
        "wrong diagnostic code"
    );
    assert!(errs[0].help.as_deref().is_some_and(|h| !h.is_empty()));
}
