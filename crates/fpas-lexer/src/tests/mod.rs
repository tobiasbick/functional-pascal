use crate::{LexError, Token, lex};

pub fn toks(input: &str) -> Vec<Token> {
    let (tokens, errors) = lex(input);
    assert!(errors.is_empty(), "unexpected errors: {errors:?}");
    tokens
        .into_iter()
        .map(|t| t.token)
        .filter(|t| *t != Token::Eof)
        .collect()
}

pub fn lex_with_errors(input: &str) -> (Vec<Token>, Vec<LexError>) {
    let (tokens, errors) = lex(input);
    let toks = tokens
        .into_iter()
        .map(|t| t.token)
        .filter(|t| *t != Token::Eof)
        .collect();
    (toks, errors)
}

mod comments;
mod directives;
mod errors;
mod identifiers;
mod integration;
mod keywords;
mod numbers;
mod preprocessor;
mod strings;
mod symbols;
