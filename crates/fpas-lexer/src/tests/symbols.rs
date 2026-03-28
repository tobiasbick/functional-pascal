use super::toks;
use crate::Token;

// ── Single-character Symbols ────────────────────────────────────

#[test]
fn colon() {
    assert_eq!(toks(":"), vec![Token::Colon]);
}

#[test]
fn semicolon() {
    assert_eq!(toks(";"), vec![Token::Semicolon]);
}

#[test]
fn comma() {
    assert_eq!(toks(","), vec![Token::Comma]);
}

#[test]
fn dot() {
    assert_eq!(toks("."), vec![Token::Dot]);
}

#[test]
fn parens() {
    assert_eq!(toks("("), vec![Token::LParen]);
    assert_eq!(toks(")"), vec![Token::RParen]);
}

#[test]
fn brackets() {
    assert_eq!(toks("["), vec![Token::LBracket]);
    assert_eq!(toks("]"), vec![Token::RBracket]);
}

#[test]
fn arithmetic() {
    assert_eq!(toks("+"), vec![Token::Plus]);
    assert_eq!(toks("-"), vec![Token::Minus]);
    assert_eq!(toks("*"), vec![Token::Star]);
    assert_eq!(toks("/"), vec![Token::Slash]);
}

#[test]
fn equal() {
    assert_eq!(toks("="), vec![Token::Equal]);
}

#[test]
fn less() {
    assert_eq!(toks("<"), vec![Token::Less]);
}

#[test]
fn greater() {
    assert_eq!(toks(">"), vec![Token::Greater]);
}

// ── Multi-character Symbols ─────────────────────────────────────

#[test]
fn colon_assign() {
    assert_eq!(toks(":="), vec![Token::ColonAssign]);
}

#[test]
fn dot_dot() {
    assert_eq!(toks(".."), vec![Token::DotDot]);
}

#[test]
fn not_equal() {
    assert_eq!(toks("<>"), vec![Token::NotEqual]);
}

#[test]
fn less_equal() {
    assert_eq!(toks("<="), vec![Token::LessEqual]);
}

#[test]
fn greater_equal() {
    assert_eq!(toks(">="), vec![Token::GreaterEqual]);
}

// ── Maximal Munch ───────────────────────────────────────────────

#[test]
fn colon_then_equal_with_space() {
    assert_eq!(toks(": ="), vec![Token::Colon, Token::Equal]);
}

#[test]
fn dot_then_dot_with_space() {
    assert_eq!(toks(". ."), vec![Token::Dot, Token::Dot]);
}

#[test]
fn less_then_greater_with_space() {
    assert_eq!(toks("< >"), vec![Token::Less, Token::Greater]);
}

#[test]
fn all_multi_char_adjacent() {
    assert_eq!(
        toks(":=..<><=>= "),
        vec![
            Token::ColonAssign,
            Token::DotDot,
            Token::NotEqual,
            Token::LessEqual,
            Token::GreaterEqual,
        ]
    );
}

// ── Symbol sequences ────────────────────────────────────────────

#[test]
fn paren_not_comment() {
    assert_eq!(
        toks("( foo )"),
        vec![Token::LParen, Token::Ident("foo".into()), Token::RParen]
    );
}

#[test]
fn brackets_with_content() {
    assert_eq!(
        toks("[0]"),
        vec![Token::LBracket, Token::Integer(0), Token::RBracket]
    );
}

#[test]
fn complex_expression() {
    assert_eq!(
        toks("a + b * c"),
        vec![
            Token::Ident("a".into()),
            Token::Plus,
            Token::Ident("b".into()),
            Token::Star,
            Token::Ident("c".into()),
        ]
    );
}

#[test]
fn assignment_expression() {
    assert_eq!(
        toks("x := 42"),
        vec![
            Token::Ident("x".into()),
            Token::ColonAssign,
            Token::Integer(42),
        ]
    );
}

#[test]
fn comparison_chain() {
    assert_eq!(
        toks("a <= b"),
        vec![
            Token::Ident("a".into()),
            Token::LessEqual,
            Token::Ident("b".into()),
        ]
    );
}
