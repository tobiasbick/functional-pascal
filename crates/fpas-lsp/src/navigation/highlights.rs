//! Document-highlight conversion.

use fpas_language_service::{
    DocumentHighlight as ServiceHighlight, DocumentSnapshot, HighlightKind,
};
use tower_lsp_server::ls_types::{DocumentHighlight, DocumentHighlightKind};

use crate::convert::PositionConversionError;

use super::span_range;

pub(crate) fn document_highlight(
    snapshot: &DocumentSnapshot,
    highlight: ServiceHighlight,
) -> Result<DocumentHighlight, PositionConversionError> {
    let kind = match highlight.kind {
        HighlightKind::Declaration => DocumentHighlightKind::TEXT,
        HighlightKind::Read => DocumentHighlightKind::READ,
        HighlightKind::Write => DocumentHighlightKind::WRITE,
    };
    Ok(DocumentHighlight {
        range: span_range(snapshot, highlight.span)?,
        kind: Some(kind),
    })
}
