use super::super::toks;
use crate::Token;

#[test]
fn record_type_and_construction() {
    let src = "\
type Point = record
  X: real;
  Y: real;
end record;
var P: Point := Point( X := 0.0, Y := 5.0 );";

    assert_eq!(
        toks(src),
        vec![
            Token::Type,
            Token::Ident("Point".into()),
            Token::Equal,
            Token::Record,
            Token::Ident("X".into()),
            Token::Colon,
            Token::Ident("real".into()),
            Token::Semicolon,
            Token::Ident("Y".into()),
            Token::Colon,
            Token::Ident("real".into()),
            Token::Semicolon,
            Token::End,
            Token::Record,
            Token::Semicolon,
            Token::Var,
            Token::Ident("P".into()),
            Token::Colon,
            Token::Ident("Point".into()),
            Token::ColonAssign,
            Token::Ident("Point".into()),
            Token::LParen,
            Token::Ident("X".into()),
            Token::ColonAssign,
            Token::Real(0.0),
            Token::Comma,
            Token::Ident("Y".into()),
            Token::ColonAssign,
            Token::Real(5.0),
            Token::RParen,
            Token::Semicolon,
        ]
    );
}
