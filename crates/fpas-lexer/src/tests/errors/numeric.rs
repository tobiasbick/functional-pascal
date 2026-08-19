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

#[test]
fn exponent_rejects_leading_digit_separator() {
    let (tokens, errors) = lex_with_errors("1.0e_3");
    assert!(!errors.is_empty());
    assert!(!tokens.contains(&Token::Real(1000.0)));
    assert!(errors[0].message.contains("exponent"));
}

#[test]
fn signed_exponent_rejects_leading_digit_separator() {
    let (tokens, errors) = lex_with_errors("1.0e+_3");
    assert!(!errors.is_empty());
    assert!(!tokens.contains(&Token::Real(1000.0)));
    assert!(errors[0].message.contains("exponent"));
}

#[test]
fn real_literal_overflow_reports_error_and_no_token() {
    use fpas_diagnostics::codes::LEX_REAL_LITERAL_OVERFLOW;

    let (toks, errs) = lex_with_errors("1.0e9999");
    assert!(toks.is_empty(), "non-finite reals must not produce a token");
    assert_eq!(errs.len(), 1);
    assert_eq!(errs[0].code, LEX_REAL_LITERAL_OVERFLOW);
}

#[test]
fn real_literal_underflow_reports_error_and_no_token() {
    use fpas_diagnostics::codes::LEX_REAL_LITERAL_OVERFLOW;

    let (toks, errs) = lex_with_errors("1.0e-9999");
    assert!(
        toks.is_empty(),
        "underflowed non-zero reals must not produce a token"
    );
    assert_eq!(errs.len(), 1);
    assert_eq!(errs[0].code, LEX_REAL_LITERAL_OVERFLOW);
}

#[test]
fn zero_mantissa_with_huge_negative_exponent_is_zero() {
    assert_eq!(super::super::toks("0.0e-9999"), vec![Token::Real(0.0)]);
}

#[test]
fn double_underscore_in_decimal_is_rejected() {
    use fpas_diagnostics::codes::LEX_INVALID_DIGIT_SEPARATOR;

    let (toks, errs) = lex_with_errors("1__2");
    assert_eq!(errs.len(), 1);
    assert_eq!(errs[0].code, LEX_INVALID_DIGIT_SEPARATOR);
    // Recovery keeps digits after the bad separator run.
    assert_eq!(toks, vec![Token::Integer(12)]);
}

#[test]
fn double_underscore_in_hex_is_rejected() {
    use fpas_diagnostics::codes::LEX_INVALID_DIGIT_SEPARATOR;

    let (toks, errs) = lex_with_errors("$A__B");
    assert_eq!(errs.len(), 1);
    assert_eq!(errs[0].code, LEX_INVALID_DIGIT_SEPARATOR);
    assert_eq!(toks, vec![Token::Integer(0xAB)]);
}
