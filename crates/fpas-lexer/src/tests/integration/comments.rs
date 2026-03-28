use super::super::toks;
use crate::Token;

#[test]
fn mutable_var_with_comments() {
    let src = "\
mutable var { counter } Count: integer := 0; // start at zero";
    assert_eq!(
        toks(src),
        vec![
            Token::Mutable,
            Token::Var,
            Token::Ident("Count".into()),
            Token::Colon,
            Token::Ident("integer".into()),
            Token::ColonAssign,
            Token::Integer(0),
            Token::Semicolon,
        ]
    );
}
