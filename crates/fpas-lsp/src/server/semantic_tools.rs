//! Full semantic-token and deterministic quick-fix request handlers.

use tower_lsp_server::jsonrpc::{Error, Result};
use tower_lsp_server::ls_types::{
    CodeActionKind, CodeActionOrCommand, CodeActionParams, CodeActionResponse,
    SemanticTokensParams, SemanticTokensResult,
};

use super::{Backend, errors};
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
            .map_err(errors::request)?;
        let tokens = semantic_tools::semantic_tokens(&result.snapshot, &result.value)
            .map_err(conversion_error)?;
        Ok(Some(tokens.into()))
    }

    pub(super) async fn code_action_request(
        &self,
        params: CodeActionParams,
    ) -> Result<Option<CodeActionResponse>> {
        if !self.supports_document_changes() {
            return Ok(Some(Vec::new()));
        }
        let uri = params.text_document.uri;
        let path =
            file_uri_to_path(&uri).map_err(|error| Error::invalid_params(error.to_string()))?;
        let mut actions = Vec::new();
        if kind_requested(params.context.only.as_deref(), &CodeActionKind::QUICKFIX) {
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
                .map_err(errors::request)?;
            actions = result
                .actions
                .into_iter()
                .map(|(action, diagnostic)| {
                    semantic_tools::code_action(&result.snapshot, uri.clone(), action, diagnostic)
                        .map(CodeActionOrCommand::CodeAction)
                })
                .collect::<std::result::Result<Vec<_>, _>>()
                .map_err(conversion_error)?;
        }
        if self.supports_refactor_application() {
            use crate::refactor_requests::{EditorRecipe, RefactorParams};
            let recipes = [
                (
                    "Extract variable",
                    CodeActionKind::REFACTOR_EXTRACT,
                    EditorRecipe::ExtractVariable {
                        uri: uri.clone(),
                        range: params.range,
                    },
                ),
                (
                    "Organize imports",
                    CodeActionKind::SOURCE_ORGANIZE_IMPORTS,
                    EditorRecipe::OrganizeImports { uri },
                ),
            ];
            for (title, kind, recipe) in recipes {
                if !kind_requested(params.context.only.as_deref(), &kind)
                    || (kind == CodeActionKind::REFACTOR_EXTRACT
                        && params.range.start == params.range.end)
                {
                    continue;
                }
                let prepared = self
                    .documents
                    .refactor(RefactorParams::Prepare {
                        version: 1,
                        context: None,
                        recipe: Box::new(recipe),
                    })
                    .await
                    .map_err(errors::request)?;
                if prepared.status != fpas_language_service::RefactorStatus::Prepared {
                    continue;
                }
                if let Some(handle) = prepared.handle {
                    actions.push(CodeActionOrCommand::CodeAction(
                        tower_lsp_server::ls_types::CodeAction {
                            title: title.into(),
                            kind: Some(kind),
                            command: Some(tower_lsp_server::ls_types::Command {
                                title: title.into(),
                                command: "fpas.applyRefactor".into(),
                                arguments: Some(vec![serde_json::Value::String(handle)]),
                            }),
                            ..Default::default()
                        },
                    ));
                }
            }
        }
        Ok(Some(actions))
    }
}

fn kind_requested(only: Option<&[CodeActionKind]>, offered: &CodeActionKind) -> bool {
    only.is_none_or(|kinds| {
        kinds.iter().any(|kind| {
            kind.as_str().is_empty()
                || kind == offered
                || offered
                    .as_str()
                    .strip_prefix(kind.as_str())
                    .is_some_and(|suffix| suffix.starts_with('.'))
        })
    })
}

fn ranges_overlap(
    requested: tower_lsp_server::ls_types::Range,
    diagnostic: tower_lsp_server::ls_types::Range,
) -> bool {
    if requested.start == requested.end {
        return if diagnostic.start == diagnostic.end {
            requested.start == diagnostic.start
        } else {
            diagnostic.start <= requested.start && requested.start < diagnostic.end
        };
    }
    if diagnostic.start == diagnostic.end {
        return requested.start <= diagnostic.start && diagnostic.start < requested.end;
    }
    requested.start < diagnostic.end && diagnostic.start < requested.end
}

fn conversion_error(error: impl std::fmt::Debug) -> Error {
    tracing::warn!(?error, "cannot convert semantic tooling result");
    Error::internal_error()
}

#[cfg(test)]
mod tests {
    use tower_lsp_server::ls_types::{Position, Range};

    use super::ranges_overlap;

    fn range(start: u32, end: u32) -> Range {
        Range::new(Position::new(0, start), Position::new(0, end))
    }

    #[test]
    fn non_empty_ranges_use_half_open_overlap() {
        assert!(ranges_overlap(range(1, 4), range(2, 3)));
        assert!(ranges_overlap(range(2, 3), range(1, 4)));
        assert!(!ranges_overlap(range(1, 2), range(2, 3)));
        assert!(!ranges_overlap(range(3, 4), range(1, 3)));
    }

    #[test]
    fn empty_request_is_a_cursor_position() {
        assert!(ranges_overlap(range(2, 2), range(2, 4)));
        assert!(ranges_overlap(range(3, 3), range(2, 4)));
        assert!(!ranges_overlap(range(4, 4), range(2, 4)));
        assert!(ranges_overlap(range(2, 2), range(2, 2)));
    }

    #[test]
    fn empty_diagnostic_belongs_to_a_non_empty_request() {
        assert!(ranges_overlap(range(1, 4), range(1, 1)));
        assert!(ranges_overlap(range(1, 4), range(3, 3)));
        assert!(!ranges_overlap(range(1, 4), range(4, 4)));
    }
}
