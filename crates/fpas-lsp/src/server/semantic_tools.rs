//! Full semantic-token and deterministic quick-fix request handlers.

use tower_lsp_server::jsonrpc::{Error, Result};
use tower_lsp_server::ls_types::{
    CodeActionKind, CodeActionOrCommand, CodeActionParams, CodeActionResponse,
    SemanticTokensParams, SemanticTokensResult,
};

use super::Backend;
use crate::convert::file_uri_to_path;
use crate::semantic_tools;

impl Backend {
    pub(super) async fn semantic_tokens_full_request(
        &self,
        params: SemanticTokensParams,
    ) -> Result<Option<SemanticTokensResult>> {
        let path = file_uri_to_path(&params.text_document.uri)
            .map_err(|error| Error::invalid_params(error.to_string()))?;
        let result = self
            .documents
            .semantic_tokens_open(&path)
            .await
            .map_err(invalid_params)?;
        let tokens = semantic_tools::semantic_tokens(&result.snapshot, &result.value)
            .map_err(conversion_error)?;
        Ok(Some(tokens.into()))
    }

    pub(super) async fn code_action_request(
        &self,
        params: CodeActionParams,
    ) -> Result<Option<CodeActionResponse>> {
        if !quick_fixes_requested(params.context.only.as_deref()) {
            return Ok(Some(Vec::new()));
        }
        let uri = params.text_document.uri;
        let path =
            file_uri_to_path(&uri).map_err(|error| Error::invalid_params(error.to_string()))?;
        let diagnostics = params
            .context
            .diagnostics
            .into_iter()
            .filter(|diagnostic| ranges_overlap(params.range, diagnostic.range))
            .collect();
        let result = self
            .documents
            .code_actions_open(&path, diagnostics)
            .await
            .map_err(invalid_params)?;
        let actions = result
            .actions
            .into_iter()
            .map(|(action, diagnostic)| {
                semantic_tools::code_action(&result.snapshot, uri.clone(), action, diagnostic)
                    .map(CodeActionOrCommand::CodeAction)
            })
            .collect::<std::result::Result<Vec<_>, _>>()
            .map_err(conversion_error)?;
        Ok(Some(actions))
    }
}

fn quick_fixes_requested(only: Option<&[CodeActionKind]>) -> bool {
    only.is_none_or(|kinds| {
        kinds.iter().any(|kind| {
            kind.as_str().is_empty() || kind.as_str() == CodeActionKind::QUICKFIX.as_str()
        })
    })
}

fn ranges_overlap(
    requested: tower_lsp_server::ls_types::Range,
    diagnostic: tower_lsp_server::ls_types::Range,
) -> bool {
    requested.start <= diagnostic.end && diagnostic.start <= requested.end
}

fn invalid_params(error: impl std::fmt::Display) -> Error {
    Error::invalid_params(error.to_string())
}

fn conversion_error(error: impl std::fmt::Debug) -> Error {
    tracing::warn!(?error, "cannot convert semantic tooling result");
    Error::internal_error()
}
