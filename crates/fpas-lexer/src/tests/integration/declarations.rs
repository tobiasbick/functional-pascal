use super::super::toks;
use crate::Token;

#[test]
fn record_type_and_construction() {
    let src = "\
type Point = record
  X: real;
  Y: real;
end;
var P: Point := record X := 0.0; Y := 5.0 end;";

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
            Token::Semicolon,
            Token::Var,
            Token::Ident("P".into()),
            Token::Colon,
            Token::Ident("Point".into()),
            Token::ColonAssign,
            Token::Record,
            Token::Ident("X".into()),
            Token::ColonAssign,
            Token::Real(0.0),
            Token::Semicolon,
            Token::Ident("Y".into()),
            Token::ColonAssign,
            Token::Real(5.0),
            Token::End,
            Token::Semicolon,
        ]
    );
}
