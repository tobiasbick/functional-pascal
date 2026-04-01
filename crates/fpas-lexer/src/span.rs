#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Span {
    pub offset: usize,
    pub length: usize,
    pub line: u32,
    pub column: u32,
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
