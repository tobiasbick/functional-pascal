use fpas_lexer::Span;

/// Span for synthetic std/builtin AST nodes that have no source location.
#[must_use]
pub(crate) fn synthetic_span() -> Span {
    Span {
        offset: 0,
        length: 0,
        line: 1,
        column: 1,
        source_id: 0,
    }
}
