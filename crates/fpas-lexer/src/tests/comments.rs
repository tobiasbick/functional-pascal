use super::toks;
use crate::Token;

// ── Brace Comments ──────────────────────────────────────────────

#[test]
fn brace_comment() {
    assert_eq!(toks("{ comment } 42"), vec![Token::Integer(42)]);
}

#[test]
fn brace_comment_empty() {
    assert_eq!(toks("{} 42"), vec![Token::Integer(42)]);
}

#[test]
fn brace_comment_multi_line() {
    assert_eq!(toks("{ line1\nline2 } 42"), vec![Token::Integer(42)]);
}

#[test]
fn brace_compiler_directive_is_comment() {
    assert_eq!(toks("{$ifdef TEST} 42"), vec![Token::Integer(42)]);
}

#[test]
fn brace_does_not_nest() {
    // { outer { inner } ← first } closes the comment
    // so " still open }" is source code, not comment
    assert_eq!(toks("{ outer { inner } 42"), vec![Token::Integer(42)]);
}

// ── Paren Comments ──────────────────────────────────────────────

#[test]
fn paren_comment() {
    assert_eq!(toks("(* comment *) 42"), vec![Token::Integer(42)]);
}

#[test]
fn paren_comment_empty() {
    assert_eq!(toks("(**) 42"), vec![Token::Integer(42)]);
}

#[test]
fn paren_comment_with_star() {
    assert_eq!(toks("(* * *) 42"), vec![Token::Integer(42)]);
}

#[test]
fn paren_comment_multi_line() {
    assert_eq!(toks("(* line1\nline2 *) 42"), vec![Token::Integer(42)]);
}

// ── Line Comments ───────────────────────────────────────────────

#[test]
fn line_comment() {
    assert_eq!(toks("// comment\n42"), vec![Token::Integer(42)]);
}

#[test]
fn line_comment_at_eof() {
    assert_eq!(toks("42 // trailing"), vec![Token::Integer(42)]);
}

#[test]
fn line_comment_empty() {
    assert_eq!(toks("//\n42"), vec![Token::Integer(42)]);
}

#[test]
fn line_comment_crlf() {
    assert_eq!(toks("// comment\r\n42"), vec![Token::Integer(42)]);
}

// ── Mixed Comments ──────────────────────────────────────────────

#[test]
fn adjacent_comments() {
    assert_eq!(toks("{ a }{ b } 42"), vec![Token::Integer(42)]);
}

#[test]
fn all_comment_types() {
    assert_eq!(
        toks("{ brace } (* paren *) // line\n42"),
        vec![Token::Integer(42)]
    );
}

#[test]
fn comment_between_tokens() {
    assert_eq!(
        toks("42 { skip } 43"),
        vec![Token::Integer(42), Token::Integer(43)]
    );
}

#[test]
fn paren_comment_does_not_eat_lparen() {
    // '(' not followed by '*' is LParen, not start of comment
    assert_eq!(
        toks("(42)"),
        vec![Token::LParen, Token::Integer(42), Token::RParen]
    );
}

#[test]
fn slash_not_comment() {
    // '/' not followed by '/' is Slash symbol
    assert_eq!(
        toks("4 / 2"),
        vec![Token::Integer(4), Token::Slash, Token::Integer(2)]
    );
}
