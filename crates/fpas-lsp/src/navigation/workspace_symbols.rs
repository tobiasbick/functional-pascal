//! Workspace-symbol conversion with qualified owner context.

use tower_lsp_server::ls_types::{SymbolInformation, SymbolTag};

use crate::navigation_requests::WorkspaceSymbolDocument;

use super::{NavigationConversionError, location, symbol_kind};

pub(crate) fn workspace_symbol(
    document: WorkspaceSymbolDocument,
) -> Result<SymbolInformation, NavigationConversionError> {
    let container_name = document
        .location
        .symbol
        .qualified_name
        .strip_suffix(&format!(".{}", document.location.symbol.name))
        .map(str::to_owned);
    #[expect(
        deprecated,
        reason = "LSP 3.17 keeps the deprecated field in the wire structure"
    )]
    Ok(SymbolInformation {
        name: document.location.symbol.name.clone(),
        kind: symbol_kind(document.location.symbol.kind),
        tags: None::<Vec<SymbolTag>>,
        deprecated: None,
        location: location(&document.snapshot, &document.location)?,
        container_name,
    })
}
