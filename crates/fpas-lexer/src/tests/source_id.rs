use crate::{Token, lex_with_source_id};

#[test]
fn lex_with_source_id_on_token_spans() {
    let (tokens, errs) = lex_with_source_id("x", 7);
    assert!(errs.is_empty());
    let ident = tokens
        .iter()
        .find(|t| matches!(t.token, Token::Ident(_)))
        .expect("identifier token");
    assert_eq!(ident.span.source_id, 7);
}

#[test]
fn lex_with_source_id_on_lexer_errors() {
    let (_, errs) = lex_with_source_id("$", 99);
    assert_eq!(errs.len(), 1);
    assert_eq!(errs[0].span.source_id, 99);
}
