//! Operators, strings and comments from other languages: one diagnostic each, the FPAS token
//! where one exists, and no false positives.
//!
//! **Documentation:** `docs/pascal/tools/diagnostics.md` (Diagnostics for other-language spellings)

use super::super::{lex_with_errors, toks};
use crate::Token;
use fpas_diagnostics::codes::LEX_FOREIGN_TOKEN;

/// Asserts one foreign-token diagnostic for `found` and the recovered tokens.
fn assert_foreign(source: &str, found: &str, replacement: &str, tokens: &[Token]) {
    let (lexed, errors) = lex_with_errors(source);
    assert_eq!(errors.len(), 1, "{source}: {errors:#?}");
    let error = &errors[0];
    assert_eq!(error.code, LEX_FOREIGN_TOKEN, "{source}");
    assert_eq!(error.found.as_deref(), Some(found), "{source}");
    assert_eq!(error.expected.as_deref(), Some(replacement), "{source}");
    assert!(
        error
            .help
            .as_deref()
            .is_some_and(|help| help.ends_with(&format!("FPAS writes `{replacement}`."))),
        "{source}: {error:#?}"
    );
    assert_eq!(lexed, tokens, "{source}");
}

#[test]
fn foreign_operators_become_their_fpas_tokens() {
    let a = || Token::Ident("A".to_owned());
    let b = || Token::Ident("B".to_owned());
    for (source, found, replacement, operator) in [
        ("A && B", "&&", "and", Token::And),
        ("A || B", "||", "or", Token::Or),
        ("A != B", "!=", "<>", Token::NotEqual),
        ("A == B", "==", "=", Token::Equal),
        ("A /= B", "/=", "<>", Token::NotEqual),
        ("A => B", "=>", ":=", Token::ColonAssign),
    ] {
        assert_foreign(source, found, replacement, &[a(), operator, b()]);
    }
    assert_foreign("!A", "!", "not", &[Token::Not, a()]);
}

#[test]
fn pointer_and_address_operators_are_dropped() {
    assert_foreign("@A", "@", "F", &[Token::Ident("A".to_owned())]);
    assert_foreign(
        "^integer",
        "^",
        "Option of T",
        &[Token::Ident("integer".to_owned())],
    );
}

#[test]
fn double_quoted_string_becomes_a_string_literal() {
    assert_foreign(
        "\"it's\"",
        "\"it's\"",
        "'it''s'",
        &[Token::Str("it's".to_owned())],
    );
}

#[test]
fn ada_comments_at_line_start_or_after_terminators_are_skipped() {
    assert_foreign("-- note\n1", "-- note", "// note", &[Token::Integer(1)]);
    assert_foreign(
        "1; -- note",
        "-- note",
        "// note",
        &[Token::Integer(1), Token::Semicolon],
    );
}

#[test]
fn valid_tokens_with_the_same_characters_are_not_reported() {
    assert_eq!(
        toks("X - -1 >= 2 <> 3 / 4 = 5 // comment"),
        vec![
            Token::Ident("X".to_owned()),
            Token::Minus,
            Token::Minus,
            Token::Integer(1),
            Token::GreaterEqual,
            Token::Integer(2),
            Token::NotEqual,
            Token::Integer(3),
            Token::Slash,
            Token::Integer(4),
            Token::Equal,
            Token::Integer(5),
        ]
    );
    assert_eq!(
        toks("'say \"hi\"'"),
        vec![Token::Str("say \"hi\"".to_owned())]
    );
}
