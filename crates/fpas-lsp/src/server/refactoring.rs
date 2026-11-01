//! Native editor preparation and command execution; the client owns application and undo.

use super::{Backend, errors};
use crate::refactor_requests::{RefactorParams, RefactorResponse};
use tower_lsp_server::{
    jsonrpc::{Error, Result},
    ls_types::ExecuteCommandParams,
};

impl Backend {
    pub(crate) async fn refactor_request(
        &self,
        params: RefactorParams,
    ) -> Result<RefactorResponse> {
        if !self.supports_document_changes() {
            return Err(Error::invalid_params(
                "Refactoring requires versioned documentChanges support.",
            ));
        }
        self.documents
            .refactor(params)
            .await
            .map_err(errors::request)
    }

    pub(super) async fn execute_refactor_command(
        &self,
        params: ExecuteCommandParams,
    ) -> Result<Option<serde_json::Value>> {
        if params.command != "fpas.applyRefactor" || !self.supports_refactor_application() {
            return Err(Error::invalid_params(
                "The requested editor refactoring application route is unavailable.",
            ));
        }
        let [handle] = params.arguments.as_slice() else {
            return Err(Error::invalid_params(
                "fpas.applyRefactor requires one session-local handle.",
            ));
        };
        let handle = handle
            .as_str()
            .filter(|s| s.len() <= 256)
            .ok_or_else(|| Error::invalid_params("Invalid refactoring handle."))?;
        let response = self
            .documents
            .refactor(RefactorParams::Revalidate {
                version: 1,
                handle: handle.into(),
            })
            .await
            .map_err(errors::request)?;
        let edit = response.applicable_edit().map_err(errors::request)?;
        let applied = self.client.apply_edit(edit).await?;
        if !applied.applied {
            let mut error = Error::invalid_params(
                applied
                    .failure_reason
                    .unwrap_or_else(|| "The editor rejected the refactoring edit.".into()),
            );
            error.data = Some(
                serde_json::json!({"code":"editor_rejected", "failed_change":applied.failed_change}),
            );
            return Err(error);
        }
        Ok(Some(serde_json::json!({"status":"applied"})))
    }
}
