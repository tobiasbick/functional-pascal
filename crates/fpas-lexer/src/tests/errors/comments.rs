use super::super::lex_with_errors;
use fpas_diagnostics::codes::LEX_INVALID_COMMENT_FORM;

#[test]
fn brace_form_is_rejected_and_closed_form_recovers() {
    let (tokens, errors) = lex_with_errors("1 { not a comment } 2");
    assert_eq!(
        tokens,
        vec![crate::Token::Integer(1), crate::Token::Integer(2)]
    );
    assert_invalid_comment(&errors, "{...}");
}

#[test]
fn paren_form_is_rejected_and_closed_form_recovers() {
    let (tokens, errors) = lex_with_errors("1 (* not a comment *) 2");
    assert_eq!(
        tokens,
        vec![crate::Token::Integer(1), crate::Token::Integer(2)]
    );
    assert_invalid_comment(&errors, "(*...*)");
}

#[test]
fn unterminated_invalid_forms_report_once_and_consume_to_eof() {
    for source in ["1 { not closed", "1 (* not closed"] {
        let (tokens, errors) = lex_with_errors(source);
        assert_eq!(tokens, vec![crate::Token::Integer(1)], "{source:?}");
        assert_eq!(errors.len(), 1, "{source:?}: {errors:?}");
        assert_eq!(errors[0].code, LEX_INVALID_COMMENT_FORM);
    }
}

#[test]
fn invalid_forms_are_not_collected_as_comments() {
    for source in ["{ invalid }", "(* invalid *)"] {
        let (_, comments, errors) = crate::lex_with_comments(source);
        assert!(comments.is_empty(), "{source:?}");
        assert_eq!(errors.len(), 1, "{source:?}");
        assert_eq!(errors[0].code, LEX_INVALID_COMMENT_FORM);
    }
}

fn assert_invalid_comment(errors: &[crate::LexError], form: &str) {
    assert_eq!(errors.len(), 1, "{errors:?}");
    assert_eq!(errors[0].code, LEX_INVALID_COMMENT_FORM);
    assert!(errors[0].message.contains(form));
    assert_eq!(
        errors[0].help.as_deref(),
        Some("Use `// comment`. For multiple lines, prefix each line with `//`.")
    );
}
