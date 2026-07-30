//! LSP request handlers for symbols, hover, definitions, and completion.

use tower_lsp_server::jsonrpc::{Error, Result};
use tower_lsp_server::ls_types::{
    CompletionParams, CompletionResponse, DocumentSymbolParams, DocumentSymbolResponse,
    GotoDefinitionParams, GotoDefinitionResponse, Hover, HoverParams,
};

use super::Backend;
use crate::convert::file_uri_to_path;
use crate::navigation;

impl Backend {
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

    pub(super) async fn completion_request(
        &self,
        params: CompletionParams,
    ) -> Result<Option<CompletionResponse>> {
        let text = params.text_document_position;
        let path = request_path(&text.text_document.uri)?;
        let result = self
            .documents
            .completions_open(&path, text.position)
            .await
            .map_err(invalid_params)?;
        Ok(Some(CompletionResponse::Array(
            result
                .value
                .into_iter()
                .map(navigation::completion)
                .collect(),
        )))
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
