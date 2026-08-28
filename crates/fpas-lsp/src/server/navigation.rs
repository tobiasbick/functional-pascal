//! LSP request handlers for symbols, navigation, references, and rename.

use std::collections::BTreeMap;

use tower_lsp_server::jsonrpc::{Error, Result};
use tower_lsp_server::ls_types::request::{GotoTypeDefinitionParams, GotoTypeDefinitionResponse};
use tower_lsp_server::ls_types::{
    DocumentChanges, DocumentHighlight, DocumentHighlightParams, DocumentSymbolParams,
    DocumentSymbolResponse, GotoDefinitionParams, GotoDefinitionResponse, Hover, HoverParams,
    Location, OneOf, OptionalVersionedTextDocumentIdentifier, PrepareRenameResponse,
    ReferenceParams, RenameParams, SelectionRange, SelectionRangeParams, TextDocumentEdit,
    TextDocumentPositionParams, TextEdit, Uri, WorkspaceEdit, WorkspaceSymbolParams,
    WorkspaceSymbolResponse,
};

use super::{Backend, errors};
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
            .map_err(errors::request)?
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
            .map_err(errors::request)?;
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
            .map_err(errors::request)?;
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
            .map_err(errors::request)?;
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
            .map_err(errors::request)?;
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
            .map_err(errors::request)?;
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
            .map_err(errors::request)?;
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
            .map_err(errors::request)?;
        let locations = references
            .iter()
            .map(navigation::reference_location)
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
            .map_err(errors::request)?;
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
        if !self.supports_document_changes() {
            return Ok(None);
        }
        let text = params.text_document_position;
        let path = request_path(&text.text_document.uri)?;
        let edits = self
            .documents
            .rename_open(&path, text.position, &params.new_name)
            .await
            .map_err(errors::request)?;
        let mut changes = BTreeMap::<String, (Uri, Option<i32>, Vec<TextEdit>)>::new();
        for edit in edits {
            let (uri, version, text_edit) =
                navigation::rename_edit(edit).map_err(conversion_error)?;
            changes
                .entry(uri.to_string())
                .or_insert_with(|| (uri, version, Vec::new()))
                .2
                .push(text_edit);
        }
        let edits = changes
            .into_values()
            .map(|(uri, version, edits)| TextDocumentEdit {
                text_document: OptionalVersionedTextDocumentIdentifier { uri, version },
                edits: edits.into_iter().map(OneOf::Left).collect(),
            })
            .collect();
        Ok(Some(WorkspaceEdit {
            document_changes: Some(DocumentChanges::Edits(edits)),
            ..WorkspaceEdit::default()
        }))
    }
}

fn request_path(uri: &tower_lsp_server::ls_types::Uri) -> Result<std::path::PathBuf> {
    file_uri_to_path(uri).map_err(|error| Error::invalid_params(error.to_string()))
}

fn conversion_error(error: impl std::fmt::Debug) -> Error {
    tracing::warn!(?error, "cannot convert navigation result");
    Error::internal_error()
}
