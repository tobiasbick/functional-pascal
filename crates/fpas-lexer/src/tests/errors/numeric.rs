use super::super::lex_with_errors;
use crate::Token;

#[test]
fn dollar_alone() {
    let (toks, errs) = lex_with_errors("$");
    assert!(toks.is_empty());
    assert_eq!(errs.len(), 1);
    assert!(errs[0].message.contains("hexadecimal"));
}

#[test]
fn dollar_non_hex() {
    let (toks, errs) = lex_with_errors("$ZZ");
    assert_eq!(errs.len(), 1);
    assert_eq!(toks, vec![Token::Ident("ZZ".into())]);
}

#[test]
fn integer_overflow() {
    let (toks, errs) = lex_with_errors("99999999999999999999");
    assert!(toks.is_empty());
    assert_eq!(errs.len(), 1);
    assert!(errs[0].message.contains("too large"));
}

#[test]
fn hex_overflow() {
    let (toks, errs) = lex_with_errors("$FFFFFFFFFFFFFFFF");
    assert!(toks.is_empty());
    assert_eq!(errs.len(), 1);
}

#[test]
fn invalid_numeric_exponent_reports_explicit_help() {
    let (toks, errs) = lex_with_errors("1.0e");
    assert!(toks.is_empty());
    assert_eq!(errs.len(), 1);
    assert!(errs[0].message.contains("exponent"));
    assert!(
        errs[0]
            .help
            .as_deref()
            .is_some_and(|hint| hint.contains("1.0e3"))
    );
}
