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

#[test]
fn ada_concatenation_becomes_plus() {
    assert_foreign(
        "A & B",
        "&",
        "+",
        &[
            Token::Ident("A".to_owned()),
            Token::Plus,
            Token::Ident("B".to_owned()),
        ],
    );
}

#[test]
fn ada_attributes_are_skipped_with_their_fpas_form() {
    for (source, found, replacement) in [
        ("Integer'Image (N)", "'Image", "Conv.IntToStr(X)"),
        ("Float'Image (X)", "'Image", "Conv.RealToStr(X)"),
        ("Items'Length", "'Length", "Items.Length()"),
        ("Items'Last", "'Last", "Items.Length() - 1"),
    ] {
        let (lexed, errors) = lex_with_errors(source);
        assert_eq!(errors.len(), 1, "{source}: {errors:#?}");
        assert_eq!(errors[0].code, LEX_FOREIGN_TOKEN);
        assert_eq!(errors[0].found.as_deref(), Some(found), "{source}");
        assert_eq!(errors[0].expected.as_deref(), Some(replacement), "{source}");
        assert!(
            !lexed.iter().any(|token| matches!(token, Token::Str(_))),
            "{source}"
        );
    }
}

#[test]
fn strings_after_operators_or_spaces_are_not_attributes() {
    assert_eq!(
        toks("X + 'a' #65'b' 'it''s'"),
        vec![
            Token::Ident("X".to_owned()),
            Token::Plus,
            Token::Str("a".to_owned()),
            Token::Str("Ab".to_owned()),
            Token::Str("it's".to_owned()),
        ]
    );
}
