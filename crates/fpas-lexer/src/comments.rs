//! Comment tokens with source spans for formatters and tooling.
//!
//! **Documentation:** `docs/pascal/tools/fmt-style.md#comments`

use crate::Span;

/// A comment occurrence in source, with its exact span.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceComment {
    /// Exact byte range and start position in the source passed to the lexer.
    pub span: Span,
}

impl SourceComment {
    /// Returns the comment text as it appears in `source` (including delimiters).
    ///
    /// `source` must be the exact source snapshot from which this comment was collected. Returns
    /// `None` when the public span fields overflow, lie outside `source`, or split a UTF-8 scalar.
    #[must_use]
    pub fn text<'a>(&self, source: &'a str) -> Option<&'a str> {
        self.span.text(source)
    }

    /// Returns the byte offset one past the last comment byte, or `None` on overflow.
    #[must_use]
    pub fn end_offset(&self) -> Option<usize> {
        self.span.end_offset()
    }

    /// Returns `true` when non-whitespace code appears on the same line before this comment.
    ///
    /// `LF`, `CRLF`, and bare `CR` are recognized as line endings. Returns `None` when the comment
    /// span is not valid for the exact `source` snapshot from which it was collected.
    #[must_use]
    pub fn is_end_of_line(&self, source: &str) -> Option<bool> {
        self.text(source)?;
        let prefix = source.get(..self.span.offset)?;
        let line_start = prefix.rfind(['\n', '\r']).map_or(0, |index| index + 1);
        let before_comment = source.get(line_start..self.span.offset)?;
        Some(before_comment.chars().any(|ch| !ch.is_whitespace()))
    }
}
