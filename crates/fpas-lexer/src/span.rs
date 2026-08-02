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

impl From<Span> for fpas_diagnostics::SourceSpan {
    fn from(span: Span) -> Self {
        Self::new_with_source(
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
            offset: span.offset,
            length: span.length,
            line: span.line,
            column: span.column,
            source_id: span.source_id,
        }
    }
}
