use crate::{Token, lex};

#[test]
fn token_end_position_tracks_unicode_and_crlf() {
    let source = "'é\r\nx'";
    let (tokens, errors) = lex(source);
    assert!(errors.is_empty(), "errors: {errors:#?}");

    let token = &tokens[0];
    assert!(matches!(token.token, Token::Str(_)));
    assert_eq!(token.end.offset, source.len());
    assert_eq!(token.end.line, 2);
    assert_eq!(token.end.column, 3);
}
