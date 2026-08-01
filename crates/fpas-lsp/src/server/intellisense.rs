//! LSP handlers for completion, lazy resolution, and signature help.

use tower_lsp_server::jsonrpc::{Error, Result};
use tower_lsp_server::ls_types::{
    CompletionItem, CompletionParams, CompletionResponse, SignatureHelp, SignatureHelpParams,
};

use super::Backend;
use crate::convert::file_uri_to_path;
use crate::intellisense;

impl Backend {
    pub(super) async fn completion_request(
        &self,
        params: CompletionParams,
    ) -> Result<Option<CompletionResponse>> {
        let text = params.text_document_position;
        let path = file_uri_to_path(&text.text_document.uri)
            .map_err(|error| Error::invalid_params(error.to_string()))?;
        let result = self
            .documents
            .completions_open(&path, text.position)
            .await
            .map_err(invalid_params)?;
        let items = result
            .value
            .into_iter()
            .map(|candidate| intellisense::completion_item(&result.snapshot, candidate))
            .collect::<std::result::Result<Vec<_>, _>>()
            .map_err(conversion_error)?;
        Ok(Some(CompletionResponse::Array(items)))
    }

    pub(super) async fn completion_resolve_request(
        &self,
        item: CompletionItem,
    ) -> Result<CompletionItem> {
        let Some((path, declaration_offset)) = intellisense::resolve_identity(&item) else {
            return Ok(item);
        };
        let documentation = self
            .documents
            .completion_documentation(&path, declaration_offset)
            .await
            .map_err(invalid_params)?;
        Ok(intellisense::resolve_completion_item(item, documentation))
    }

    pub(super) async fn signature_help_request(
        &self,
        params: SignatureHelpParams,
    ) -> Result<Option<SignatureHelp>> {
        let text = params.text_document_position_params;
        let path = file_uri_to_path(&text.text_document.uri)
            .map_err(|error| Error::invalid_params(error.to_string()))?;
        let result = self
            .documents
            .signature_help_open(&path, text.position)
            .await
            .map_err(invalid_params)?;
        Ok(result.value.map(intellisense::signature_help))
    }
}

fn invalid_params(error: impl std::fmt::Display) -> Error {
    Error::invalid_params(error.to_string())
}

fn conversion_error(error: impl std::fmt::Debug) -> Error {
    tracing::warn!(?error, "cannot convert IntelliSense result");
    Error::internal_error()
}
