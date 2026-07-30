//! Hierarchical LSP document-symbol conversion.

use fpas_language_service::{DocumentSnapshot, DocumentSymbol};
use tower_lsp_server::ls_types::DocumentSymbol as LspDocumentSymbol;

use super::{NavigationConversionError, span_range, symbol_kind};

pub(crate) fn document_symbols(
    snapshot: &DocumentSnapshot,
    symbols: Vec<DocumentSymbol>,
) -> Result<Vec<LspDocumentSymbol>, NavigationConversionError> {
    symbols
        .into_iter()
        .map(|symbol| document_symbol(snapshot, symbol))
        .collect()
}

fn document_symbol(
    snapshot: &DocumentSnapshot,
    symbol: DocumentSymbol,
) -> Result<LspDocumentSymbol, NavigationConversionError> {
    #[expect(
        deprecated,
        reason = "LSP 3.17 keeps the deprecated field in the wire structure"
    )]
    Ok(LspDocumentSymbol {
        name: symbol.name,
        detail: Some(symbol.detail),
        kind: symbol_kind(symbol.kind),
        tags: None,
        deprecated: None,
        range: span_range(snapshot, symbol.full_span)?,
        selection_range: span_range(snapshot, symbol.selection_span)?,
        children: if symbol.children.is_empty() {
            None
        } else {
            Some(
                symbol
                    .children
                    .into_iter()
                    .map(|child| document_symbol(snapshot, child))
                    .collect::<Result<_, _>>()?,
            )
        },
    })
}
