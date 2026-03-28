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
