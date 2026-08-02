//! LSP-synchronized access to completion and signature assistance.

use std::path::Path;

use fpas_language_service::{CompletionCandidate, NavigationResult, SignatureHelp};
use tower_lsp_server::ls_types::Position;

use crate::convert::position_to_byte_offset;
use crate::documents::{DocumentRequestError, SynchronizedDocuments, require_open, tasks};
use crate::intellisense::CompletionResolveIdentity;

impl SynchronizedDocuments {
    pub(crate) async fn completions_open(
        &self,
        path: &Path,
        position: Position,
    ) -> Result<NavigationResult<Vec<CompletionCandidate>>, DocumentRequestError> {
        let path = path.to_path_buf();
        tasks::run(&self.service, move |service, cancellation| {
            cancellation.check()?;
            let snapshot = require_open(service, &path)?;
            let offset = position_to_byte_offset(&snapshot, position)?;
            let result = service.completions(&path, offset)?;
            cancellation.check()?;
            Ok(result)
        })
        .await
    }

    pub(crate) async fn completion_documentation(
        &self,
        identity: CompletionResolveIdentity,
    ) -> Result<Option<String>, DocumentRequestError> {
        tasks::run(&self.service, move |service, cancellation| {
            cancellation.check()?;
            let snapshot = service.snapshot(&identity.path)?;
            if snapshot.revision() != identity.source_revision {
                return Ok(None);
            }
            let documentation = service.completion_documentation(
                &identity.path,
                identity.declaration_offset,
                &identity.qualified_name,
            )?;
            cancellation.check()?;
            Ok(documentation)
        })
        .await
    }

    pub(crate) async fn signature_help_open(
        &self,
        path: &Path,
        position: Position,
    ) -> Result<NavigationResult<Option<SignatureHelp>>, DocumentRequestError> {
        let path = path.to_path_buf();
        tasks::run(&self.service, move |service, cancellation| {
            cancellation.check()?;
            let snapshot = require_open(service, &path)?;
            let offset = position_to_byte_offset(&snapshot, position)?;
            let result = service.signature_help(&path, offset)?;
            cancellation.check()?;
            Ok(result)
        })
        .await
    }
}
