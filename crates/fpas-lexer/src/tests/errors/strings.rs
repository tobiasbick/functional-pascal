use super::super::lex_with_errors;
use crate::Token;

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
fn char_code_out_of_range() {
    let (toks, errs) = lex_with_errors("#256");
    assert_eq!(errs.len(), 1);
    assert!(errs[0].message.contains("out of range"));
    assert_eq!(toks.len(), 1);
}

#[test]
fn char_code_very_large() {
    let (_, errs) = lex_with_errors("#999999");
    assert_eq!(errs.len(), 1);
}

#[test]
fn char_code_decimal_overflow_u32() {
    let (_, errs) = lex_with_errors("#99999999999999999999");
    assert_eq!(errs.len(), 1);
    assert!(errs[0].message.contains("too large"));
}

#[test]
fn char_code_error_has_explicit_help() {
    let (_, errs) = lex_with_errors("#");
    assert_eq!(errs.len(), 1);
    assert!(
        errs[0]
            .help
            .as_deref()
            .is_some_and(|hint| hint.contains("0..255"))
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
