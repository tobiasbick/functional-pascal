use super::super::toks;
use crate::Token;

#[test]
fn if_then_else() {
    let src = "if X > 0 then Y := 1 else Y := -1";
    assert_eq!(
        toks(src),
        vec![
            Token::If,
            Token::Ident("X".into()),
            Token::Greater,
            Token::Integer(0),
            Token::Then,
            Token::Ident("Y".into()),
            Token::ColonAssign,
            Token::Integer(1),
            Token::Else,
            Token::Ident("Y".into()),
            Token::ColonAssign,
            Token::Minus,
            Token::Integer(1),
        ]
    );
}

#[test]
fn for_loop() {
    let src = "for I: integer := 0 to 9 do WriteLn(I)";
    assert_eq!(
        toks(src),
        vec![
            Token::For,
            Token::Ident("I".into()),
            Token::Colon,
            Token::Ident("integer".into()),
            Token::ColonAssign,
            Token::Integer(0),
            Token::To,
            Token::Integer(9),
            Token::Do,
            Token::Ident("WriteLn".into()),
            Token::LParen,
            Token::Ident("I".into()),
            Token::RParen,
        ]
    );
}

#[test]
fn case_statement() {
    let src = "\
case X of
  0..9: WriteLn('digit');
  10: WriteLn('ten')
end";

    assert_eq!(
        toks(src),
        vec![
            Token::Case,
            Token::Ident("X".into()),
            Token::Of,
            Token::Integer(0),
            Token::DotDot,
            Token::Integer(9),
            Token::Colon,
            Token::Ident("WriteLn".into()),
            Token::LParen,
            Token::Str("digit".into()),
            Token::RParen,
            Token::Semicolon,
            Token::Integer(10),
            Token::Colon,
            Token::Ident("WriteLn".into()),
            Token::LParen,
            Token::Str("ten".into()),
            Token::RParen,
            Token::End,
        ]
    );
}
