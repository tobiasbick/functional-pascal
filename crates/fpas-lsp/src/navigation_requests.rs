//! LSP-synchronized access to Phase 09 navigation features.

use std::path::Path;

use fpas_language_service::{
    DocumentHighlight, DocumentSymbol, HoverInfo, NavigationResult, ReferenceLocation, RenameEdit,
    RenameTarget,
};
use tower_lsp_server::ls_types::Position;

use crate::convert::position_to_byte_offset;
use crate::documents::{
    DefinitionDocument, DocumentRequestError, SelectionDocument, SynchronizedDocuments,
    WorkspaceSymbolDocument, require_open, tasks,
};

impl SynchronizedDocuments {
    pub(crate) async fn document_symbols_open(
        &self,
        path: &Path,
    ) -> Result<NavigationResult<Vec<DocumentSymbol>>, DocumentRequestError> {
        let path = path.to_path_buf();
        tasks::run(&self.service, move |service, cancellation| {
            cancellation.check()?;
            require_open(service, &path)?;
            let result = service.document_symbols(&path)?;
            cancellation.check()?;
            Ok(result)
        })
        .await
    }

    pub(crate) async fn hover_open(
        &self,
        path: &Path,
        position: Position,
    ) -> Result<NavigationResult<Option<HoverInfo>>, DocumentRequestError> {
        let path = path.to_path_buf();
        tasks::run(&self.service, move |service, cancellation| {
            cancellation.check()?;
            let snapshot = require_open(service, &path)?;
            let offset = position_to_byte_offset(&snapshot, position)?;
            let result = service.hover(&path, offset)?;
            cancellation.check()?;
            Ok(result)
        })
        .await
    }

    pub(crate) async fn definitions_open(
        &self,
        path: &Path,
        position: Position,
    ) -> Result<Vec<DefinitionDocument>, DocumentRequestError> {
        let path = path.to_path_buf();
        tasks::run(&self.service, move |service, cancellation| {
            cancellation.check()?;
            let snapshot = require_open(service, &path)?;
            let offset = position_to_byte_offset(&snapshot, position)?;
            let result = service.definitions(&path, offset)?;
            definition_documents(service, result.value, cancellation)
        })
        .await
    }

    pub(crate) async fn workspace_symbols(
        &self,
        query: &str,
    ) -> Result<Vec<WorkspaceSymbolDocument>, DocumentRequestError> {
        let query = query.to_owned();
        tasks::run(&self.service, move |service, cancellation| {
            cancellation.check()?;
            let locations = service.workspace_symbols(&query)?;
            let mut symbols = Vec::with_capacity(locations.len());
            for location in locations {
                cancellation.check()?;
                symbols.push(WorkspaceSymbolDocument {
                    snapshot: service.snapshot(&location.path)?,
                    location,
                });
            }
            Ok(symbols)
        })
        .await
    }

    pub(crate) async fn document_highlights_open(
        &self,
        path: &Path,
        position: Position,
    ) -> Result<NavigationResult<Vec<DocumentHighlight>>, DocumentRequestError> {
        let path = path.to_path_buf();
        tasks::run(&self.service, move |service, cancellation| {
            cancellation.check()?;
            let snapshot = require_open(service, &path)?;
            let offset = position_to_byte_offset(&snapshot, position)?;
            let result = service.document_highlights(&path, offset)?;
            cancellation.check()?;
            Ok(result)
        })
        .await
    }

    pub(crate) async fn type_definitions_open(
        &self,
        path: &Path,
        position: Position,
    ) -> Result<Vec<DefinitionDocument>, DocumentRequestError> {
        let path = path.to_path_buf();
        tasks::run(&self.service, move |service, cancellation| {
            cancellation.check()?;
            let snapshot = require_open(service, &path)?;
            let offset = position_to_byte_offset(&snapshot, position)?;
            let result = service.type_definitions(&path, offset)?;
            let mut definitions = Vec::with_capacity(result.value.len());
            for location in result.value {
                cancellation.check()?;
                definitions.push(DefinitionDocument {
                    snapshot: service.snapshot(&location.path)?,
                    location,
                });
            }
            Ok(definitions)
        })
        .await
    }

    pub(crate) async fn selection_ranges_open(
        &self,
        path: &Path,
        positions: &[Position],
    ) -> Result<SelectionDocument, DocumentRequestError> {
        let path = path.to_path_buf();
        let positions = positions.to_vec();
        tasks::run(&self.service, move |service, cancellation| {
            cancellation.check()?;
            let snapshot = require_open(service, &path)?;
            let offsets = positions
                .iter()
                .map(|position| position_to_byte_offset(&snapshot, *position))
                .collect::<Result<Vec<_>, _>>()?;
            let (snapshot, ranges) = service.selection_ranges(&path, &offsets)?;
            cancellation.check()?;
            Ok(SelectionDocument { snapshot, ranges })
        })
        .await
    }

    pub(crate) async fn references_open(
        &self,
        path: &Path,
        position: Position,
        include_declaration: bool,
    ) -> Result<Vec<ReferenceLocation>, DocumentRequestError> {
        let path = path.to_path_buf();
        tasks::run(&self.service, move |service, cancellation| {
            let snapshot = require_open(service, &path)?;
            let offset = position_to_byte_offset(&snapshot, position)?;
            let result = service.references_with_cancellation(
                &path,
                offset,
                include_declaration,
                cancellation,
            )?;
            cancellation.check()?;
            Ok(result.value)
        })
        .await
    }

    pub(crate) async fn prepare_rename_open(
        &self,
        path: &Path,
        position: Position,
    ) -> Result<NavigationResult<Option<RenameTarget>>, DocumentRequestError> {
        let path = path.to_path_buf();
        tasks::run(&self.service, move |service, cancellation| {
            cancellation.check()?;
            let snapshot = require_open(service, &path)?;
            let offset = position_to_byte_offset(&snapshot, position)?;
            let result = service.prepare_rename(&path, offset)?;
            cancellation.check()?;
            Ok(result)
        })
        .await
    }

    pub(crate) async fn rename_open(
        &self,
        path: &Path,
        position: Position,
        new_name: &str,
    ) -> Result<Vec<RenameEdit>, DocumentRequestError> {
        let path = path.to_path_buf();
        let new_name = new_name.to_owned();
        tasks::run(&self.service, move |service, cancellation| {
            let snapshot = require_open(service, &path)?;
            let offset = position_to_byte_offset(&snapshot, position)?;
            let result =
                service.rename_with_cancellation(&path, offset, &new_name, cancellation)?;
            cancellation.check()?;
            Ok(result.value)
        })
        .await
    }
}

fn definition_documents(
    service: &mut fpas_language_service::LanguageService,
    locations: Vec<fpas_language_service::SymbolLocation>,
    cancellation: &fpas_language_service::CancellationToken,
) -> Result<Vec<DefinitionDocument>, DocumentRequestError> {
    let mut definitions = Vec::with_capacity(locations.len());
    for location in locations {
        cancellation.check()?;
        definitions.push(DefinitionDocument {
            snapshot: service.snapshot(&location.path)?,
            location,
        });
    }
    Ok(definitions)
}
