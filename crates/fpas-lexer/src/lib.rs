#![cfg_attr(
    test,
    expect(
        clippy::approx_constant,
        reason = "lexer tests assert exact Pascal source literals such as 3.14"
    )
)]

mod error;
mod lexer;
mod preprocessor;
mod span;
mod token;

pub use error::LexError;
pub use preprocessor::{DefineSet, preprocess};
pub use span::Span;
pub use token::Token;

#[derive(Debug, Clone, PartialEq)]
pub struct SpannedToken {
    pub token: Token,
    pub span: Span,
}

pub fn lex(source: &str) -> (Vec<SpannedToken>, Vec<LexError>) {
    lexer::Lexer::new(source).tokenize()
}

#[cfg(test)]
mod tests;
