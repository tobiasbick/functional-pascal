//! UTF-8 lexer for Functional Pascal source files.
//!
//! Identifiers are ASCII-only and case-insensitive for keyword recognition. One leading UTF-8 BOM
//! is accepted; later `U+FEFF` values are reported as unexpected characters. Token and comment
//! spans use byte offsets plus one-based lines and Unicode-scalar columns. `LF`, `CRLF`, and bare
//! `CR` are logical line endings.
//!
//! Lexical errors do not stop scanning. Every recovery path consumes input, invalid identifier-like
//! sequences produce one diagnostic, and every token stream ends with [`Token::Eof`].
//!
//! # Example
//!
//! ```
//! use fpas_lexer::{Token, lex};
//!
//! let source = "var café := 1;";
//! let (tokens, errors) = lex(source);
//!
//! assert_eq!(tokens[0].token, Token::Var);
//! assert_eq!(tokens[0].span.text(source), Some("var"));
//! assert_eq!(errors.len(), 1);
//! assert!(matches!(tokens.last().map(|token| &token.token), Some(Token::Eof)));
//! ```

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

pub use comments::SourceComment;
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

/// Lexes `source`, returning recovered tokens and ordered diagnostics.
///
/// The token vector always ends with [`Token::Eof`], including when diagnostics were emitted.
/// Comments are discarded; use [`lex_with_comments`] when their spans are required.
#[must_use]
pub fn lex(source: &str) -> (Vec<SpannedToken>, Vec<LexError>) {
    let (tokens, _, errors) = lex_with_comments(source);
    (tokens, errors)
}

/// Lexes `source` and returns comment spans alongside recovered tokens and diagnostics.
///
/// The token vector always ends with [`Token::Eof`]. Every returned [`SourceComment`] refers to the
/// exact `source` snapshot passed to this function.
#[must_use]
pub fn lex_with_comments(source: &str) -> (Vec<SpannedToken>, Vec<SourceComment>, Vec<LexError>) {
    lexer::Lexer::new(source).tokenize_with_comments()
}

/// Returns every comment span from `source` in source order.
///
/// Each returned comment refers to the exact `source` snapshot passed to this function.
#[must_use]
pub fn collect_comments(source: &str) -> Vec<SourceComment> {
    lex_with_comments(source).1
}

/// Like [`lex_with_comments`], but attaches `source_id` to every returned span.
///
/// The identifier is opaque and lets callers associate spans with the exact source snapshot they
/// supplied. The lexer does not interpret it.
#[must_use]
pub fn lex_with_source_id(
    source: &str,
    source_id: u32,
) -> (Vec<SpannedToken>, Vec<SourceComment>, Vec<LexError>) {
    lexer::Lexer::with_source_id(source, source_id).tokenize_with_comments()
}

#[cfg(test)]
mod tests;
