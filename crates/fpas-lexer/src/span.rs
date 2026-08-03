use std::ops::Range;

/// A zero-width position in a UTF-8 source file.
///
/// Offsets count bytes. Lines and columns are one-based, and columns count Unicode scalar values.
/// `CRLF` and bare `CR` each advance to the next line just like `LF`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SourcePosition {
    /// Zero-based byte offset from the start of the source.
    pub offset: usize,
    /// One-based source line.
    pub line: u32,
    /// One-based Unicode-scalar column.
    pub column: u32,
}

/// A byte range with its one-based starting line and column.
///
/// Lexer-produced spans always identify UTF-8 boundaries within the source passed to the lexer.
/// Because the fields remain public, manually constructed spans must be checked with [`Self::text`]
/// before slicing a source.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Span {
    /// Zero-based byte offset from the start of the source.
    pub offset: usize,
    /// Length of the range in bytes.
    pub length: usize,
    /// One-based line containing the start of the range.
    pub line: u32,
    /// One-based Unicode-scalar column containing the start of the range.
    pub column: u32,
    /// Identifier of the source containing the range.
    pub source_id: u32,
}

impl Span {
    /// Returns the half-open byte range when `offset + length` does not overflow.
    #[must_use]
    pub fn byte_range(self) -> Option<Range<usize>> {
        let end = self.offset.checked_add(self.length)?;
        Some(self.offset..end)
    }

    /// Returns the spanned text when this span is valid for `source`.
    ///
    /// `source` must be the exact source snapshot associated with this span. The method validates
    /// byte bounds and UTF-8 boundaries, but a plain string cannot prove source identity.
    #[must_use]
    pub fn text(self, source: &str) -> Option<&str> {
        source.get(self.byte_range()?)
    }

    /// Returns the byte offset immediately after this span, or `None` on overflow.
    #[must_use]
    pub fn end_offset(self) -> Option<usize> {
        self.offset.checked_add(self.length)
    }

    /// Converts this span for diagnostic use, substituting a safe placeholder if it is invalid.
    ///
    /// Prefer the fallible [`fpas_diagnostics::SourceSpan::try_from`] conversion when callers can
    /// propagate validation failures. This method keeps diagnostic reporting non-panicking for
    /// malformed spans constructed through the public fields.
    #[must_use]
    pub fn diagnostic_span_or_synthetic(self) -> fpas_diagnostics::SourceSpan {
        let source_id = self.source_id;
        fpas_diagnostics::SourceSpan::try_from(self).unwrap_or_else(|_| {
            fpas_diagnostics::SourceSpan::new_with_source(0, 0, 1, 1, source_id)
        })
    }
}

impl TryFrom<Span> for fpas_diagnostics::SourceSpan {
    type Error = fpas_diagnostics::SourceSpanError;

    fn try_from(span: Span) -> Result<Self, Self::Error> {
        Self::try_new_with_source(
            span.offset,
            span.length,
            span.line,
            span.column,
            span.source_id,
        )
    }
}

impl From<fpas_diagnostics::SourceSpan> for Span {
    fn from(span: fpas_diagnostics::SourceSpan) -> Self {
        Self {
            offset: span.offset(),
            length: span.length(),
            line: span.line(),
            column: span.column(),
            source_id: span.source_id(),
        }
    }
}

#[cfg(test)]
mod tests {
    use fpas_diagnostics::{SourceLocationError, SourceSpan, SourceSpanError};

    use super::Span;

    #[test]
    fn diagnostic_span_conversion_preserves_valid_values() {
        let span = Span {
            offset: 8,
            length: 3,
            line: 2,
            column: 4,
            source_id: 7,
        };

        assert_eq!(
            SourceSpan::try_from(span),
            Ok(SourceSpan::new_with_source(8, 3, 2, 4, 7))
        );
    }

    #[test]
    fn diagnostic_span_conversion_rejects_invalid_public_fields() {
        let span = Span {
            offset: usize::MAX,
            length: 1,
            line: 0,
            column: 1,
            source_id: 7,
        };

        assert_eq!(
            SourceSpan::try_from(span),
            Err(SourceSpanError::Location(SourceLocationError::ZeroLine))
        );
    }

    #[test]
    fn diagnostic_span_fallback_is_non_panicking_and_retains_source_id() {
        let span = Span {
            offset: usize::MAX,
            length: 1,
            line: 1,
            column: 1,
            source_id: 7,
        };

        assert_eq!(
            span.diagnostic_span_or_synthetic(),
            SourceSpan::new_with_source(0, 0, 1, 1, 7)
        );
    }

    #[test]
    fn text_validates_overflow_bounds_and_utf8_boundaries() {
        let valid = Span {
            offset: 0,
            length: 2,
            line: 1,
            column: 1,
            source_id: 0,
        };
        assert_eq!(valid.text("éx"), Some("é"));

        assert_eq!(Span { offset: 1, ..valid }.text("éx"), None);
        assert_eq!(Span { length: 4, ..valid }.text("éx"), None);
        assert_eq!(
            Span {
                offset: usize::MAX,
                length: 1,
                ..valid
            }
            .text("éx"),
            None
        );
    }
}
