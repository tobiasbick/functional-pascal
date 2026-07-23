use fpas_lexer::Span;
use fpas_parser::QualifiedId;

pub(super) fn apply_qualified_id_source_id(id: &mut QualifiedId, source_id: u32) {
    apply_span(&mut id.span, source_id);
}

pub(super) fn apply_span(span: &mut Span, source_id: u32) {
    span.source_id = source_id;
}
