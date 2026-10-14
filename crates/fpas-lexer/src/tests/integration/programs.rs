use super::super::toks;
use crate::Token;

#[test]
fn hello_world_program() {
    let src = "program Hello;\nuses\n  Std.Console as Console;\nbegin\n  Console.WriteLn('Hello, World!')\nend.";

    assert_eq!(
        toks(src),
        vec![
            Token::Program,
            Token::Ident("Hello".into()),
            Token::Semicolon,
            Token::Uses,
            Token::Ident("Std".into()),
            Token::Dot,
            Token::Ident("Console".into()),
            Token::Ident("as".into()),
            Token::Ident("Console".into()),
            Token::Semicolon,
            Token::Begin,
            Token::Ident("Console".into()),
            Token::Dot,
            Token::Ident("WriteLn".into()),
            Token::LParen,
            Token::Str("Hello, World!".into()),
            Token::RParen,
            Token::End,
            Token::Dot,
        ]
    );
}

#[test]
fn function_declaration() {
    let src = "\
function Add(A: integer; B: integer): integer;
begin
  return A + B
end function;";

    assert_eq!(
        toks(src),
        vec![
            Token::Function,
            Token::Ident("Add".into()),
            Token::LParen,
            Token::Ident("A".into()),
            Token::Colon,
            Token::Ident("integer".into()),
            Token::Semicolon,
            Token::Ident("B".into()),
            Token::Colon,
            Token::Ident("integer".into()),
            Token::RParen,
            Token::Colon,
            Token::Ident("integer".into()),
            Token::Semicolon,
            Token::Begin,
            Token::Return,
            Token::Ident("A".into()),
            Token::Plus,
            Token::Ident("B".into()),
            Token::End,
            Token::Function,
            Token::Semicolon,
        ]
    );
}
