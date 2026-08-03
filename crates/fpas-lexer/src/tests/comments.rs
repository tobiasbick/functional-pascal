use super::{lex_with_errors, toks};
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
fn brace_compiler_directive_is_lex_error_not_comment() {
    // `{$...}` is not a comment; the lexer reports an error and skips the sequence.
    let (tokens, errors) = lex_with_errors("{$ifdef TEST} 42");
    assert_eq!(tokens, vec![Token::Integer(42)]);
    assert_eq!(errors.len(), 1);
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
fn bare_line_comment_at_eof_does_not_panic() {
    assert_eq!(toks("//"), Vec::<Token>::new());
    assert_eq!(toks("42 //"), vec![Token::Integer(42)]);
}

#[test]
fn bare_doc_comment_at_eof_does_not_panic() {
    assert_eq!(toks("///"), Vec::<Token>::new());
}

#[test]
fn utf8_bom_is_skipped_as_trivia() {
    assert_eq!(toks("\u{FEFF}42"), vec![Token::Integer(42)]);
    assert_eq!(toks("\u{FEFF}//\n42"), vec![Token::Integer(42)]);
}

#[test]
fn leading_bom_preserves_byte_offset_and_advances_scalar_column() {
    let source = "\u{FEFF}42";
    let (tokens, errors) = crate::lex(source);
    assert!(errors.is_empty(), "{errors:?}");
    assert_eq!(tokens[0].span.offset, '\u{FEFF}'.len_utf8());
    assert_eq!(tokens[0].span.line, 1);
    assert_eq!(tokens[0].span.column, 2);
}

#[test]
fn bom_in_the_middle_is_an_unexpected_character() {
    use fpas_diagnostics::codes::LEX_UNEXPECTED_CHARACTER;

    let (tokens, errors) = lex_with_errors("foo\u{FEFF}bar");
    assert_eq!(
        tokens,
        vec![Token::Ident("foo".into()), Token::Ident("bar".into())]
    );
    assert_eq!(errors.len(), 1);
    assert_eq!(errors[0].code, LEX_UNEXPECTED_CHARACTER);
    assert_eq!(errors[0].span.offset(), 3);
    assert_eq!(errors[0].span.length(), '\u{FEFF}'.len_utf8());
}

#[test]
fn bom_after_a_token_at_eof_is_an_unexpected_character() {
    use fpas_diagnostics::codes::LEX_UNEXPECTED_CHARACTER;

    let (tokens, errors) = lex_with_errors("x\u{FEFF}");
    assert_eq!(tokens, vec![Token::Ident("x".into())]);
    assert_eq!(errors.len(), 1);
    assert_eq!(errors[0].code, LEX_UNEXPECTED_CHARACTER);
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
        toks("{ brace } (* paren *) /// doc\n// line\n42"),
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
fn unicode_whitespace_is_skipped() {
    assert_eq!(super::toks("\u{00A0}42"), vec![crate::Token::Integer(42)]);
}

#[test]
fn slash_not_comment() {
    // '/' not followed by '/' is Slash symbol
    assert_eq!(
        toks("4 / 2"),
        vec![Token::Integer(4), Token::Slash, Token::Integer(2)]
    );
}

// ── Non-nesting and cross-style (language/basics) ──────────────────

#[test]
fn paren_comment_does_not_nest() {
    // (* outer (* inner *) ← first *) closes the comment
    assert_eq!(toks("(* outer (* inner *) 42"), vec![Token::Integer(42)]);
}

#[test]
fn line_comment_inside_brace_comment() {
    // // inside { } is just comment text, not a line comment
    assert_eq!(
        toks("{ // not a line comment } 42"),
        vec![Token::Integer(42)]
    );
}

#[test]
fn brace_inside_line_comment() {
    // { inside // is just comment text — brace does NOT start a new comment
    assert_eq!(
        toks("// { not a brace comment\n42"),
        vec![Token::Integer(42)]
    );
}

#[test]
fn paren_comment_inside_brace_comment() {
    assert_eq!(toks("{ (* still brace *) } 42"), vec![Token::Integer(42)]);
}

#[test]
fn brace_inside_paren_comment() {
    assert_eq!(toks("(* { still paren } *) 42"), vec![Token::Integer(42)]);
}

#[test]
fn line_comment_inside_paren_comment() {
    assert_eq!(toks("(* // still paren *) 42"), vec![Token::Integer(42)]);
}

#[test]
fn comment_preserves_surrounding_tokens() {
    assert_eq!(
        toks("1 { skip } + (* skip *) 2 // trailing"),
        vec![Token::Integer(1), Token::Plus, Token::Integer(2)]
    );
}
