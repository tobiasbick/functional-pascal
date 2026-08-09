use super::*;
use fpas_diagnostics::codes::LEX_COMPILER_DIRECTIVE_NOT_SUPPORTED;

#[test]
fn directive_sequence_is_error_and_emits_no_token() {
    let (tokens, errors) = lex_with_errors("{$IFDEF DEBUG}");
    assert!(tokens.is_empty());
    assert_eq!(errors.len(), 1);
    assert_eq!(errors[0].code, LEX_COMPILER_DIRECTIVE_NOT_SUPPORTED);
}

#[test]
fn directive_error_skips_sequence_and_keeps_following_tokens() {
    let (tokens, errors) = lex_with_errors("{$ENDIF} 42");
    assert_eq!(tokens, vec![Token::Integer(42)]);
    assert_eq!(errors.len(), 1);
    assert_eq!(errors[0].code, LEX_COMPILER_DIRECTIVE_NOT_SUPPORTED);
}

#[test]
fn empty_directive_braces_is_error() {
    let (tokens, errors) = lex_with_errors("{$}");
    assert!(tokens.is_empty());
    assert_eq!(errors.len(), 1);
    assert_eq!(errors[0].code, LEX_COMPILER_DIRECTIVE_NOT_SUPPORTED);
}

#[test]
fn directive_between_code_produces_error_and_preserves_neighbors() {
    let (tokens, errors) = lex_with_errors("1 {$ELSE} 2");
    assert_eq!(tokens, vec![Token::Integer(1), Token::Integer(2)]);
    assert_eq!(errors.len(), 1);
    assert_eq!(errors[0].code, LEX_COMPILER_DIRECTIVE_NOT_SUPPORTED);
}

#[test]
fn unterminated_directive_uses_directive_diagnostic() {
    let (tokens, errors) = lex_with_errors("{$IFDEF DEBUG");
    assert!(tokens.is_empty());
    assert_eq!(errors.len(), 1);
    assert_eq!(errors[0].code, LEX_COMPILER_DIRECTIVE_NOT_SUPPORTED);
}

#[test]
fn invalid_brace_form_before_directive_reports_both_sequences() {
    let (tokens, errors) = lex_with_errors("{ invalid } {$ENDIF}");
    assert!(tokens.is_empty());
    assert_eq!(errors.len(), 2);
    assert_eq!(
        errors[0].code,
        fpas_diagnostics::codes::LEX_INVALID_COMMENT_FORM
    );
    assert_eq!(errors[1].code, LEX_COMPILER_DIRECTIVE_NOT_SUPPORTED);
}
