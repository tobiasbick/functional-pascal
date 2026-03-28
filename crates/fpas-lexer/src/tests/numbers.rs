use super::toks;
use crate::Token;

// ── Integers ────────────────────────────────────────────────────

#[test]
fn integer_zero() {
    assert_eq!(toks("0"), vec![Token::Integer(0)]);
}

#[test]
fn integer_simple() {
    assert_eq!(toks("42"), vec![Token::Integer(42)]);
    assert_eq!(toks("1000000"), vec![Token::Integer(1000000)]);
}

#[test]
fn integer_underscores() {
    assert_eq!(toks("1_000_000"), vec![Token::Integer(1_000_000)]);
    assert_eq!(toks("1_0"), vec![Token::Integer(10)]);
    assert_eq!(toks("100_000_000"), vec![Token::Integer(100_000_000)]);
}

#[test]
fn integer_max() {
    assert_eq!(toks("9223372036854775807"), vec![Token::Integer(i64::MAX)]);
}

// ── Hex Integers ────────────────────────────────────────────────

#[test]
fn hex_simple() {
    assert_eq!(toks("$FF"), vec![Token::Integer(255)]);
    assert_eq!(toks("$0"), vec![Token::Integer(0)]);
    assert_eq!(toks("$10"), vec![Token::Integer(16)]);
    assert_eq!(toks("$1A"), vec![Token::Integer(26)]);
}

#[test]
fn hex_lowercase() {
    assert_eq!(toks("$ff"), vec![Token::Integer(255)]);
    assert_eq!(toks("$ab"), vec![Token::Integer(171)]);
}

#[test]
fn hex_underscores() {
    assert_eq!(toks("$FF_FF"), vec![Token::Integer(65535)]);
    assert_eq!(toks("$AB_CD_EF"), vec![Token::Integer(0xABCDEF)]);
}

// ── Real Literals ───────────────────────────────────────────────

#[test]
fn real_simple() {
    assert_eq!(toks("3.14"), vec![Token::Real(3.14)]);
    assert_eq!(toks("0.5"), vec![Token::Real(0.5)]);
    assert_eq!(toks("5.0"), vec![Token::Real(5.0)]);
    assert_eq!(toks("0.0"), vec![Token::Real(0.0)]);
}

#[test]
fn real_with_underscores() {
    assert_eq!(toks("1_000.500_1"), vec![Token::Real(1000.5001)]);
}

#[test]
fn scientific_notation() {
    assert_eq!(toks("1.5e10"), vec![Token::Real(1.5e10)]);
    assert_eq!(toks("3.0E4"), vec![Token::Real(3.0e4)]);
}

#[test]
fn scientific_with_sign() {
    assert_eq!(toks("3.0E-4"), vec![Token::Real(3.0e-4)]);
    assert_eq!(toks("1.0e+2"), vec![Token::Real(1.0e+2)]);
}

#[test]
fn scientific_zero_exponent() {
    assert_eq!(toks("1.0e0"), vec![Token::Real(1.0)]);
}

// ── Disambiguation ──────────────────────────────────────────────

#[test]
fn range_not_real() {
    assert_eq!(
        toks("0..10"),
        vec![Token::Integer(0), Token::DotDot, Token::Integer(10)]
    );
}

#[test]
fn range_with_spaces() {
    assert_eq!(
        toks("0 .. 10"),
        vec![Token::Integer(0), Token::DotDot, Token::Integer(10)]
    );
}

#[test]
fn real_then_dot() {
    assert_eq!(
        toks("3.14.x"),
        vec![Token::Real(3.14), Token::Dot, Token::Ident("x".into()),]
    );
}

#[test]
fn integer_then_dot() {
    assert_eq!(
        toks("42.x"),
        vec![Token::Integer(42), Token::Dot, Token::Ident("x".into())]
    );
}

#[test]
fn trailing_underscore_not_consumed() {
    // 42_ → Integer(42), then _ is scanned as identifier
    assert_eq!(
        toks("42_x"),
        vec![Token::Integer(42), Token::Ident("_x".into())]
    );
}

#[test]
fn multiple_numbers() {
    assert_eq!(
        toks("1 2 3"),
        vec![Token::Integer(1), Token::Integer(2), Token::Integer(3)]
    );
}
