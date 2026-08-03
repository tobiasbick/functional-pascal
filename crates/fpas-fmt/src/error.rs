//! Public formatter errors.

use std::fmt;

/// Failure while combining source text with a parsed compilation unit.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FormatError {
    /// `unit` is not the syntax tree produced by parsing `source`.
    SourceMismatch,
    /// An AST span lies outside `source` or splits a UTF-8 code point.
    InvalidSourceSpan {
        /// Start byte offset supplied by the AST.
        offset: usize,
        /// Byte length supplied by the AST.
        length: usize,
        /// Byte length of the source passed to the formatter.
        source_len: usize,
    },
}

impl fmt::Display for FormatError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::SourceMismatch => formatter.write_str(
                "the parsed unit does not match the source; parse the same source before formatting",
            ),
            Self::InvalidSourceSpan {
                offset,
                length,
                source_len,
            } => write!(
                formatter,
                "the parsed unit contains source span {offset}..{} outside the {source_len}-byte source; parse the same source before formatting",
                offset.saturating_add(*length)
            ),
        }
    }
}

impl std::error::Error for FormatError {}
