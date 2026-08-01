//! LSP request handlers for symbols, navigation, references, and rename.

use std::collections::HashMap;

use tower_lsp_server::jsonrpc::{Error, Result};
use tower_lsp_server::ls_types::request::{GotoTypeDefinitionParams, GotoTypeDefinitionResponse};
use tower_lsp_server::ls_types::{
    DocumentHighlight, DocumentHighlightParams, DocumentSymbolParams, DocumentSymbolResponse,
    GotoDefinitionParams, GotoDefinitionResponse, Hover, HoverParams, Location,
    PrepareRenameResponse, ReferenceParams, RenameParams, SelectionRange, SelectionRangeParams,
    TextDocumentPositionParams, TextEdit, Uri, WorkspaceEdit, WorkspaceSymbolParams,
    WorkspaceSymbolResponse,
};

use super::Backend;
use crate::convert::file_uri_to_path;
use crate::navigation;

impl Backend {
    pub(super) async fn workspace_symbol_request(
        &self,
        params: WorkspaceSymbolParams,
    ) -> Result<Option<WorkspaceSymbolResponse>> {
        let symbols = self
            .documents
            .workspace_symbols(&params.query)
            .await
            .map_err(invalid_params)?
            .into_iter()
            .map(navigation::workspace_symbol)
            .collect::<std::result::Result<Vec<_>, _>>()
            .map_err(conversion_error)?;
        Ok(Some(WorkspaceSymbolResponse::Flat(symbols)))
    }

    pub(super) async fn document_symbol_request(
        &self,
        params: DocumentSymbolParams,
    ) -> Result<Option<DocumentSymbolResponse>> {
        let path = request_path(&params.text_document.uri)?;
        let result = self
            .documents
            .document_symbols_open(&path)
            .await
            .map_err(invalid_params)?;
        let symbols = navigation::document_symbols(&result.snapshot, result.value)
            .map_err(conversion_error)?;
        Ok(Some(DocumentSymbolResponse::Nested(symbols)))
    }

    pub(super) async fn hover_request(&self, params: HoverParams) -> Result<Option<Hover>> {
        let text = params.text_document_position_params;
        let path = request_path(&text.text_document.uri)?;
        let result = self
            .documents
            .hover_open(&path, text.position)
            .await
            .map_err(invalid_params)?;
        result
            .value
            .map(|value| navigation::hover(&result.snapshot, value))
            .transpose()
            .map_err(conversion_error)
    }

    pub(super) async fn definition_request(
        &self,
        params: GotoDefinitionParams,
    ) -> Result<Option<GotoDefinitionResponse>> {
        let text = params.text_document_position_params;
        let path = request_path(&text.text_document.uri)?;
        let definitions = self
            .documents
            .definitions_open(&path, text.position)
            .await
            .map_err(invalid_params)?;
        let locations = definitions
            .iter()
            .map(|definition| navigation::location(&definition.snapshot, &definition.location))
            .collect::<std::result::Result<Vec<_>, _>>()
            .map_err(conversion_error)?;
        Ok((!locations.is_empty()).then_some(GotoDefinitionResponse::Array(locations)))
    }

    pub(super) async fn type_definition_request(
        &self,
        params: GotoTypeDefinitionParams,
    ) -> Result<Option<GotoTypeDefinitionResponse>> {
        let text = params.text_document_position_params;
        let path = request_path(&text.text_document.uri)?;
        let definitions = self
            .documents
            .type_definitions_open(&path, text.position)
            .await
            .map_err(invalid_params)?;
        let locations = definitions
            .iter()
            .map(|definition| navigation::location(&definition.snapshot, &definition.location))
            .collect::<std::result::Result<Vec<_>, _>>()
            .map_err(conversion_error)?;
        Ok((!locations.is_empty()).then_some(GotoDefinitionResponse::Array(locations)))
    }

    pub(super) async fn document_highlight_request(
        &self,
        params: DocumentHighlightParams,
    ) -> Result<Option<Vec<DocumentHighlight>>> {
        let text = params.text_document_position_params;
        let path = request_path(&text.text_document.uri)?;
        let result = self
            .documents
            .document_highlights_open(&path, text.position)
            .await
            .map_err(invalid_params)?;
        let highlights = result
            .value
            .into_iter()
            .map(|highlight| navigation::document_highlight(&result.snapshot, highlight))
            .collect::<std::result::Result<Vec<_>, _>>()
            .map_err(conversion_error)?;
        Ok(Some(highlights))
    }

    pub(super) async fn selection_range_request(
        &self,
        params: SelectionRangeParams,
    ) -> Result<Option<Vec<SelectionRange>>> {
        let path = request_path(&params.text_document.uri)?;
        let document = self
            .documents
            .selection_ranges_open(&path, &params.positions)
            .await
            .map_err(invalid_params)?;
        let ranges = document
            .ranges
            .into_iter()
            .map(|range| navigation::selection_range(&document.snapshot, range))
            .collect::<std::result::Result<Vec<_>, _>>()
            .map_err(conversion_error)?;
        Ok(Some(ranges))
    }

    pub(super) async fn references_request(
        &self,
        params: ReferenceParams,
    ) -> Result<Option<Vec<Location>>> {
        let text = params.text_document_position;
        let path = request_path(&text.text_document.uri)?;
        let references = self
            .documents
            .references_open(&path, text.position, params.context.include_declaration)
            .await
            .map_err(invalid_params)?;
        let locations = references
            .iter()
            .map(|reference| {
                navigation::reference_location(&reference.snapshot, &reference.location)
            })
            .collect::<std::result::Result<Vec<_>, _>>()
            .map_err(conversion_error)?;
        Ok(Some(locations))
    }

    pub(super) async fn prepare_rename_request(
        &self,
        params: TextDocumentPositionParams,
    ) -> Result<Option<PrepareRenameResponse>> {
        let path = request_path(&params.text_document.uri)?;
        let result = self
            .documents
            .prepare_rename_open(&path, params.position)
            .await
            .map_err(invalid_params)?;
        result
            .value
            .map(|target| navigation::prepare_rename(&result.snapshot, target))
            .transpose()
            .map_err(conversion_error)
    }

    pub(super) async fn rename_request(
        &self,
        params: RenameParams,
    ) -> Result<Option<WorkspaceEdit>> {
        let text = params.text_document_position;
        let path = request_path(&text.text_document.uri)?;
        let edits = self
            .documents
            .rename_open(&path, text.position, &params.new_name)
            .await
            .map_err(invalid_params)?;
        let mut changes = HashMap::<Uri, Vec<TextEdit>>::new();
        for edit in edits {
            let (uri, text_edit) =
                navigation::rename_edit(&edit.snapshot, edit.edit).map_err(conversion_error)?;
            changes.entry(uri).or_default().push(text_edit);
        }
        Ok(Some(WorkspaceEdit::new(changes)))
    }
}

fn request_path(uri: &tower_lsp_server::ls_types::Uri) -> Result<std::path::PathBuf> {
    file_uri_to_path(uri).map_err(|error| Error::invalid_params(error.to_string()))
}

fn invalid_params(error: impl std::fmt::Display) -> Error {
    Error::invalid_params(error.to_string())
}

fn conversion_error(error: impl std::fmt::Debug) -> Error {
    tracing::warn!(?error, "cannot convert navigation result");
    Error::internal_error()
}
