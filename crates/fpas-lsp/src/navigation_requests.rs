//! LSP-synchronized access to Phase 09 navigation features.

use std::path::Path;

use fpas_language_service::{DocumentHighlight, NavigationResult, SelectionRange, SymbolLocation};
use tower_lsp_server::ls_types::Position;

use crate::convert::position_to_byte_offset;
use crate::documents::{
    DefinitionDocument, DocumentRequestError, SynchronizedDocuments, require_open,
};

pub(crate) struct WorkspaceSymbolDocument {
    pub(crate) snapshot: std::sync::Arc<fpas_language_service::DocumentSnapshot>,
    pub(crate) location: SymbolLocation,
}

pub(crate) struct SelectionDocument {
    pub(crate) snapshot: std::sync::Arc<fpas_language_service::DocumentSnapshot>,
    pub(crate) ranges: Vec<SelectionRange>,
}

impl SynchronizedDocuments {
    pub(crate) async fn workspace_symbols(
        &self,
        query: &str,
    ) -> Result<Vec<WorkspaceSymbolDocument>, DocumentRequestError> {
        let mut service = self.service.lock().await;
        let locations = service.workspace_symbols(query)?;
        let mut symbols = Vec::with_capacity(locations.len());
        for location in locations {
            symbols.push(WorkspaceSymbolDocument {
                snapshot: service.snapshot(&location.path)?,
                location,
            });
        }
        Ok(symbols)
    }

    pub(crate) async fn document_highlights_open(
        &self,
        path: &Path,
        position: Position,
    ) -> Result<NavigationResult<Vec<DocumentHighlight>>, DocumentRequestError> {
        let mut service = self.service.lock().await;
        let snapshot = require_open(&service, path)?;
        let offset = position_to_byte_offset(&snapshot, position)?;
        Ok(service.document_highlights(path, offset)?)
    }

    pub(crate) async fn type_definitions_open(
        &self,
        path: &Path,
        position: Position,
    ) -> Result<Vec<DefinitionDocument>, DocumentRequestError> {
        let mut service = self.service.lock().await;
        let snapshot = require_open(&service, path)?;
        let offset = position_to_byte_offset(&snapshot, position)?;
        let result = service.type_definitions(path, offset)?;
        let mut definitions = Vec::with_capacity(result.value.len());
        for location in result.value {
            definitions.push(DefinitionDocument {
                snapshot: service.snapshot(&location.path)?,
                location,
            });
        }
        Ok(definitions)
    }

    pub(crate) async fn selection_ranges_open(
        &self,
        path: &Path,
        positions: &[Position],
    ) -> Result<SelectionDocument, DocumentRequestError> {
        let mut service = self.service.lock().await;
        let snapshot = require_open(&service, path)?;
        let offsets = positions
            .iter()
            .map(|position| position_to_byte_offset(&snapshot, *position))
            .collect::<Result<Vec<_>, _>>()?;
        let (snapshot, ranges) = service.selection_ranges(path, &offsets)?;
        Ok(SelectionDocument { snapshot, ranges })
    }
}
