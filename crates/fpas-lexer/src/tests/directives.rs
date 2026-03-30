use super::{lex_with_errors, toks};
use crate::Token;

// ── Token emission ───────────────────────────────────────────────────────────

#[test]
fn directive_emits_directive_token() {
    assert_eq!(
        toks("{$IFDEF DEBUG}"),
        vec![Token::Directive("IFDEF DEBUG".into())]
    );
}

#[test]
fn directive_trimmed_whitespace() {
    assert_eq!(
        toks("{$  DEFINE  FOO  }"),
        vec![Token::Directive("DEFINE  FOO".into())]
    );
}

#[test]
fn directive_lowercase_preserved_in_payload() {
    // The lexer should preserve case; normalisation happens in the preprocessor.
    assert_eq!(
        toks("{$ifdef debug}"),
        vec![Token::Directive("ifdef debug".into())]
    );
}

#[test]
fn directive_followed_by_code() {
    assert_eq!(
        toks("{$ENDIF} 42"),
        vec![Token::Directive("ENDIF".into()), Token::Integer(42)]
    );
}

#[test]
fn directive_between_tokens() {
    assert_eq!(
        toks("1 {$ELSE} 2"),
        vec![
            Token::Integer(1),
            Token::Directive("ELSE".into()),
            Token::Integer(2)
        ]
    );
}

#[test]
fn directive_empty_content_is_unknown() {
    // An empty `{$}` emits a directive with an empty payload.
    assert_eq!(toks("{$}"), vec![Token::Directive("".into())]);
}

#[test]
fn directive_does_not_suppress_adjacent_comment() {
    // A plain comment after a directive is still skipped.
    assert_eq!(
        toks("{$ENDIF} { regular comment } 7"),
        vec![Token::Directive("ENDIF".into()), Token::Integer(7)]
    );
}

// ── Errors ───────────────────────────────────────────────────────────────────

#[test]
fn unterminated_directive_is_an_error() {
    let (tokens, errors) = lex_with_errors("{$IFDEF DEBUG");
    assert!(tokens.is_empty(), "expected no tokens, got {tokens:?}");
    assert_eq!(errors.len(), 1);
    let msg = &errors[0].message;
    assert!(
        msg.contains("Unterminated compiler directive"),
        "unexpected message: {msg}"
    );
}

// ── Regression: plain brace comments still work ──────────────────────────────

#[test]
fn plain_brace_comment_is_still_skipped() {
    assert_eq!(toks("{ hello } 99"), vec![Token::Integer(99)]);
}

#[test]
fn brace_comment_followed_by_directive() {
    assert_eq!(
        toks("{ comment } {$ENDIF}"),
        vec![Token::Directive("ENDIF".into())]
    );
}
