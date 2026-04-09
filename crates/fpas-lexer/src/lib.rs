#![cfg_attr(
    test,
    expect(
        clippy::approx_constant,
        reason = "lexer tests assert exact Pascal source literals such as 3.14"
    )
)]

mod error;
mod lexer;
mod span;
mod token;

pub use error::LexError;
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

/// Like [`lex`], but attaches `source_id` to every token and lexer diagnostic span.
#[must_use]
pub fn lex_with_source_id(source: &str, source_id: u32) -> (Vec<SpannedToken>, Vec<LexError>) {
    lexer::Lexer::with_source_id(source, source_id).tokenize()
}

#[cfg(test)]
mod tests;
