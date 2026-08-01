//! LSP-synchronized access to completion and signature assistance.

use std::path::Path;

use fpas_language_service::{CompletionCandidate, NavigationResult, SignatureHelp};
use tower_lsp_server::ls_types::Position;

use crate::convert::position_to_byte_offset;
use crate::documents::{DocumentRequestError, SynchronizedDocuments, require_open};

impl SynchronizedDocuments {
    pub(crate) async fn completions_open(
        &self,
        path: &Path,
        position: Position,
    ) -> Result<NavigationResult<Vec<CompletionCandidate>>, DocumentRequestError> {
        let mut service = self.service.lock().await;
        let snapshot = require_open(&service, path)?;
        let offset = position_to_byte_offset(&snapshot, position)?;
        Ok(service.completions(path, offset)?)
    }

    pub(crate) async fn completion_documentation(
        &self,
        path: &Path,
        declaration_offset: usize,
    ) -> Result<Option<String>, DocumentRequestError> {
        let mut service = self.service.lock().await;
        Ok(service.completion_documentation(path, declaration_offset)?)
    }

    pub(crate) async fn signature_help_open(
        &self,
        path: &Path,
        position: Position,
    ) -> Result<NavigationResult<Option<SignatureHelp>>, DocumentRequestError> {
        let mut service = self.service.lock().await;
        let snapshot = require_open(&service, path)?;
        let offset = position_to_byte_offset(&snapshot, position)?;
        Ok(service.signature_help(path, offset)?)
    }
}
