#![cfg_attr(
    test,
    expect(
        clippy::approx_constant,
        reason = "lexer tests assert exact Pascal source literals such as 3.14"
    )
)]
mod comments;
mod error;
mod lexer;
mod span;
mod token;

pub use comments::{CommentStyle, SourceComment};
pub use error::LexError;
pub use span::{SourcePosition, Span};
pub use token::Token;

/// A lexical token together with its exact source range and end position.
#[derive(Debug, Clone, PartialEq)]
pub struct SpannedToken {
    /// Token kind and decoded payload.
    pub token: Token,
    /// Source range occupied by the token.
    pub span: Span,
    /// Position immediately after the token, before any following trivia.
    ///
    /// Unlike `span.column + span.length`, this remains exact for multi-byte Unicode and
    /// multi-line tokens.
    pub end: SourcePosition,
}

pub fn lex(source: &str) -> (Vec<SpannedToken>, Vec<LexError>) {
    let (tokens, _, errors) = lex_with_comments(source);
    (tokens, errors)
}

/// Lexes `source` and returns comment spans alongside tokens.
#[must_use]
pub fn lex_with_comments(source: &str) -> (Vec<SpannedToken>, Vec<SourceComment>, Vec<LexError>) {
    lexer::Lexer::new(source).tokenize_with_comments()
}

/// Returns every comment span from `source` (same order as [`lex_with_comments`]).
#[must_use]
pub fn collect_comments(source: &str) -> Vec<SourceComment> {
    lex_with_comments(source).1
}

/// Like [`lex`], but attaches `source_id` to token, comment, and diagnostic spans.
#[must_use]
pub fn lex_with_source_id(
    source: &str,
    source_id: u32,
) -> (Vec<SpannedToken>, Vec<SourceComment>, Vec<LexError>) {
    lexer::Lexer::with_source_id(source, source_id).tokenize_with_comments()
}

#[cfg(test)]
mod tests;
