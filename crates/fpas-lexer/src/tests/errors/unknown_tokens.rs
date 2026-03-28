use super::super::lex_with_errors;
use crate::Token;

#[test]
fn unknown_at() {
    let (toks, errs) = lex_with_errors("@");
    assert!(toks.is_empty());
    assert_eq!(errs.len(), 1);
    assert!(errs[0].message.contains('`'));
    assert!(errs[0].message.contains('@'));
}

#[test]
fn unknown_tilde() {
    let (_, errs) = lex_with_errors("~");
    assert_eq!(errs.len(), 1);
    assert!(errs[0].message.contains('~'));
}

#[test]
fn unknown_backtick() {
    let (_, errs) = lex_with_errors("`");
    assert_eq!(errs.len(), 1);
}

#[test]
fn error_then_valid_token() {
    let (toks, errs) = lex_with_errors("@ 42");
    assert_eq!(errs.len(), 1);
    assert_eq!(toks, vec![Token::Integer(42)]);
}

#[test]
fn multiple_unknown_chars() {
    let (_, errs) = lex_with_errors("@ ~ `");
    assert_eq!(errs.len(), 3);
}

#[test]
fn error_messages_have_hints() {
    let (_, errs) = lex_with_errors("@ 'unclosed $ #");
    for err in &errs {
        assert!(
            err.help
                .as_deref()
                .is_some_and(|hint| !hint.trim().is_empty()),
            "Error missing hint: {}",
            err.message
        );
    }
}

#[test]
fn error_spans_are_set() {
    let (_, errs) = lex_with_errors("  @");
    assert_eq!(errs.len(), 1);
    assert_eq!(errs[0].span.line, 1);
    assert_eq!(errs[0].span.column, 3);
}

#[test]
fn error_on_second_line() {
    let (_, errs) = lex_with_errors("\n@");
    assert_eq!(errs.len(), 1);
    assert_eq!(errs[0].span.line, 2);
    assert_eq!(errs[0].span.column, 1);
}

#[test]
fn unexpected_char_has_correct_code() {
    use fpas_diagnostics::codes::LEX_UNEXPECTED_CHARACTER;

    let (_, errs) = lex_with_errors("  @");
    assert_eq!(errs.len(), 1);
    assert_eq!(
        errs[0].code, LEX_UNEXPECTED_CHARACTER,
        "wrong diagnostic code"
    );
    assert_eq!(errs[0].span.line, 1, "wrong line");
    assert_eq!(errs[0].span.column, 3, "wrong column");
    assert!(
        errs[0].help.as_deref().is_some_and(|h| !h.is_empty()),
        "help text must be present"
    );
}
