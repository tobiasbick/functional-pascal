//! Synchronized semantic-token and code-action language-service access.

use std::path::Path;

use fpas_language_service::{
    NavigationResult, SemanticCodeAction, SemanticToken as FpasSemanticToken,
};
use tower_lsp_server::ls_types::Diagnostic;

use crate::documents::{DocumentRequestError, SynchronizedDocuments, require_open};
use crate::semantic_tools;

pub(crate) struct CodeActionResult {
    pub(crate) snapshot: std::sync::Arc<fpas_language_service::DocumentSnapshot>,
    pub(crate) actions: Vec<(SemanticCodeAction, Diagnostic)>,
}

impl SynchronizedDocuments {
    pub(crate) async fn semantic_tokens_open(
        &self,
        path: &Path,
    ) -> Result<NavigationResult<Vec<FpasSemanticToken>>, DocumentRequestError> {
        let mut service = self.service.lock().await;
        require_open(&service, path)?;
        Ok(service.semantic_tokens(path)?)
    }

    pub(crate) async fn code_actions_open(
        &self,
        path: &Path,
        diagnostics: Vec<Diagnostic>,
    ) -> Result<CodeActionResult, DocumentRequestError> {
        let mut service = self.service.lock().await;
        let snapshot = require_open(&service, path)?;
        let mut actions = Vec::new();
        for diagnostic in diagnostics {
            let identity = match semantic_tools::diagnostic_identity(&snapshot, &diagnostic) {
                Ok(Some(identity)) => identity,
                Ok(None) | Err(_) => continue,
            };
            for action in service.code_actions(path, &identity)?.value {
                actions.push((action, diagnostic.clone()));
            }
        }
        Ok(CodeActionResult { snapshot, actions })
    }
}
