//! Shared-core preparation, snapshot-bound handles and fresh editor revalidation.

mod handles;
mod model;
mod projection;
pub(crate) use handles::Handles;
pub(crate) use model::{EditorRecipe, EditorSelector, RefactorParams, RefactorResponse};

use crate::documents::{
    DocumentRequestError, SynchronizedDocuments,
    read_set::{Versions, failure},
    tasks,
};
use fpas_language_service::{REFACTOR_JSON_BYTES, RefactorStatus, SourceSession};
use std::path::{Path, PathBuf};
use tower_lsp_server::ls_types::Uri;

impl SynchronizedDocuments {
    pub(crate) async fn refactor(
        &self,
        params: RefactorParams,
    ) -> Result<RefactorResponse, DocumentRequestError> {
        if serde_json::to_vec(&params).map_or(true, |s| s.len() > REFACTOR_JSON_BYTES) {
            return Err(failure(
                "invalid_request",
                "Refactoring request exceeds its byte limit.",
            ));
        }
        let saved = match &params {
            RefactorParams::Revalidate { version: 1, handle } => {
                Some(self.refactor_handles.lock().await.get(handle)?)
            }
            RefactorParams::Prepare { version: 1, .. } => None,
            _ => {
                return Err(failure(
                    "invalid_request",
                    "Unsupported refactoring protocol version.",
                ));
            }
        };
        let (mut response, plan, versions, inventory) = tasks::run(&self.service, move |service, cancellation| {
            if saved.as_ref().is_some_and(|s| !s.versions.matches(service)) {
                return Err(failure("stale_plan", "A participating buffer or workspace configuration changed after preparation."));
            }
            let input = match &params {
                RefactorParams::Prepare { context, recipe, .. } => context.as_ref().map(uri_path).transpose()?
                    .or_else(|| match recipe.as_ref() {
                        EditorRecipe::Rename { selector: EditorSelector::Reference { target_ref }, .. } | EditorRecipe::AddParameter { selector: EditorSelector::Reference { target_ref }, .. } => Some(PathBuf::from(&target_ref.context.input)),
                        _ => None,
                    })
                    .or_else(|| service.workspace().manifest_path().map(Path::to_path_buf))
                    .or(recipe_input(recipe)?).ok_or_else(|| failure("invalid_request", "Symbol selection requires a manifest context."))?,
                RefactorParams::Revalidate { .. } => PathBuf::from(&saved.as_ref().ok_or_else(|| failure("stale_plan", "Missing refactoring handle."))?.plan.context.input),
            };
            let library = service.standard_library_root().map(Path::to_path_buf);
            let mut session = SourceSession::load(&input, library.as_deref(), &service.documents().open_snapshots(), cancellation)?;
            let versions = Versions::capture(service, &session);
            let mut prepared = match params {
                RefactorParams::Prepare { recipe, .. } => {
                    let recipe = projection::recipe(&mut session, *recipe)?;
                    session.prepare_refactor(&recipe, cancellation)?
                }
                RefactorParams::Revalidate { .. } => session.revalidate_refactor(&saved.as_ref().ok_or_else(|| failure("stale_plan", "Missing refactoring handle."))?.plan, cancellation)?,
            };
            let response = RefactorResponse {
                version: 1, status: prepared.status, context: session.context().clone(), snapshot: session.snapshot().clone(),
                handle: None, edit: prepared.changes.take().map(|c| projection::workspace_edit(c.edits)).transpose()?,
                coverage: model::RefactorCoverage { analysis_complete: prepared.status != RefactorStatus::Incomplete,
                    reasons: prepared.blockers.iter().map(|b| b.code.clone()).collect() },
                blockers: prepared.blockers, diagnostics: crate::source_tools_requests::diagnostics(&prepared.diagnostics), limitations: prepared.limitations,
            };
            if serde_json::to_vec(&response).map_or(true, |s| s.len() + 256 > REFACTOR_JSON_BYTES) {
                return Err(failure("plan_limit", "The complete refactoring result exceeds the editor byte limit."));
            }
            cancellation.check()?;
            if !session.verify()? { return Err(failure("source_changed", "Refactoring inputs changed during preparation.")); }
            Ok((response, prepared.plan, versions, session.inventory().clone()))
        }).await?;
        if let Some(plan) = plan {
            response.handle = Some(self.refactor_handles.lock().await.insert(handles::Saved {
                plan,
                versions: versions.clone(),
            })?);
        }
        self.verify_read_set(&versions, inventory).await?;
        Ok(response)
    }
}

impl RefactorResponse {
    pub(crate) fn applicable_edit(
        self,
    ) -> Result<tower_lsp_server::ls_types::WorkspaceEdit, DocumentRequestError> {
        if matches!(
            self.status,
            RefactorStatus::Prepared | RefactorStatus::NoChange
        ) {
            return self
                .edit
                .ok_or_else(|| failure("invalid_plan", "Checked refactoring edits are missing."));
        }
        Err(failure(
            if self.status == RefactorStatus::Incomplete {
                "analysis_incomplete"
            } else {
                "unsupported_refactoring"
            },
            &self
                .blockers
                .iter()
                .map(|b| format!("{}: {}", b.code, b.message))
                .collect::<Vec<_>>()
                .join("\n"),
        ))
    }
}

fn recipe_input(recipe: &EditorRecipe) -> Result<Option<PathBuf>, DocumentRequestError> {
    match recipe {
        EditorRecipe::Rename {
            selector: EditorSelector::Position { uri, .. },
            ..
        }
        | EditorRecipe::AddParameter {
            selector: EditorSelector::Position { uri, .. },
            ..
        }
        | EditorRecipe::ExtractVariable { uri, .. }
        | EditorRecipe::OrganizeImports { uri } => uri_path(uri).map(Some),
        EditorRecipe::Rename {
            selector: EditorSelector::Reference { target_ref },
            ..
        }
        | EditorRecipe::AddParameter {
            selector: EditorSelector::Reference { target_ref },
            ..
        } => Ok(Some(PathBuf::from(&target_ref.context.input))),
        EditorRecipe::Rename {
            selector: EditorSelector::Symbol { .. },
            ..
        }
        | EditorRecipe::AddParameter {
            selector: EditorSelector::Symbol { .. },
            ..
        } => Ok(None),
    }
}

fn uri_path(uri: &Uri) -> Result<PathBuf, DocumentRequestError> {
    crate::convert::file_uri_to_path(uri).map_err(|e| failure("invalid_request", &e.to_string()))
}
