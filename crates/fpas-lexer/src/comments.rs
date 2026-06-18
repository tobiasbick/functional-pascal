//! Comment tokens with source spans for formatters and tooling.
//!
//! **Documentation:** `docs/pascal/tools/fmt-style.md#comments`

use crate::Span;

/// Comment lexical style.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CommentStyle {
    /// Line comment whose text begins with `///` (doc line).
    DocLine,
    /// Brace comment `{ ... }`.
    Brace,
    /// Parenthesis comment `(* ... *)`.
    Paren,
    /// Ordinary line comment `//` (not doc).
    Line,
}

/// A comment occurrence in source, with its exact span.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceComment {
    pub style: CommentStyle,
    pub span: Span,
}

impl SourceComment {
    /// Returns the comment text as it appears in `source` (including delimiters).
    #[must_use]
    pub fn text<'a>(&self, source: &'a str) -> &'a str {
        &source[self.span.offset..self.end_offset()]
    }

    /// Byte offset one past the last comment byte.
    #[must_use]
    pub fn end_offset(&self) -> usize {
        self.span.offset + self.span.length
    }

    /// Whether v2 formatting may re-attach this comment before a declaration.
    #[must_use]
    pub fn is_preservable(&self) -> bool {
        matches!(
            self.style,
            CommentStyle::DocLine | CommentStyle::Brace | CommentStyle::Paren
        )
    }

    /// Returns `true` when non-whitespace code appears on the same line before this comment.
    #[must_use]
    pub fn is_end_of_line(&self, source: &str) -> bool {
        let line_start = source[..self.span.offset]
            .rfind('\n')
            .map(|index| index + 1)
            .unwrap_or(0);
        !source[line_start..self.span.offset]
            .chars()
            .all(char::is_whitespace)
    }
}
