//! Synchronized semantic-token and code-action language-service access.

use std::path::Path;

use fpas_language_service::{NavigationResult, SemanticToken as FpasSemanticToken};
use tower_lsp_server::ls_types::Diagnostic;

use crate::documents::{
    CodeActionResult, DocumentRequestError, SynchronizedDocuments, require_open, tasks,
};
use crate::semantic_tools;

impl SynchronizedDocuments {
    pub(crate) async fn semantic_tokens_open(
        &self,
        path: &Path,
    ) -> Result<NavigationResult<Vec<FpasSemanticToken>>, DocumentRequestError> {
        let path = path.to_path_buf();
        tasks::run(&self.service, move |service, cancellation| {
            cancellation.check()?;
            require_open(service, &path)?;
            let result = service.semantic_tokens(&path)?;
            cancellation.check()?;
            Ok(result)
        })
        .await
    }

    pub(crate) async fn code_actions_open(
        &self,
        path: &Path,
        diagnostics: Vec<Diagnostic>,
    ) -> Result<CodeActionResult, DocumentRequestError> {
        let path = path.to_path_buf();
        tasks::run(&self.service, move |service, cancellation| {
            cancellation.check()?;
            let snapshot = require_open(service, &path)?;
            let mut actions = Vec::new();
            for diagnostic in diagnostics {
                cancellation.check()?;
                let identity = match semantic_tools::diagnostic_identity(&snapshot, &diagnostic) {
                    Ok(Some(identity)) => identity,
                    Ok(None) | Err(_) => continue,
                };
                for action in service.code_actions(&path, &identity)?.value {
                    actions.push((action, diagnostic.clone()));
                }
            }
            Ok(CodeActionResult { snapshot, actions })
        })
        .await
    }
}
