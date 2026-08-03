/// A zero-width position in a source file.
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
}
