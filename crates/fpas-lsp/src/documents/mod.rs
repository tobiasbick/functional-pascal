//! Synchronized primary document state and isolated query snapshots.

mod errors;
mod lifecycle;
mod results;
pub(crate) mod tasks;

use std::path::{Path, PathBuf};
use std::sync::Arc;

use fpas_language_service::{DocumentSnapshot, LanguageService};
use tokio::sync::Mutex;

pub(crate) use errors::{DocumentRequestError, DocumentSyncError};
pub(crate) use results::{
    CodeActionResult, DefinitionDocument, FormattedDocument, SelectionDocument,
    SynchronizedDocument, WorkspaceSymbolDocument,
};

/// Primary editor state shared by lifecycle notifications and snapshot capture.
pub(crate) struct SynchronizedDocuments {
    pub(crate) service: Arc<Mutex<LanguageService>>,
}

impl SynchronizedDocuments {
    pub(crate) fn new(initial_root: PathBuf) -> Self {
        Self {
            service: Arc::new(Mutex::new(LanguageService::load(&initial_root))),
        }
    }
}

pub(crate) fn require_open(
    service: &LanguageService,
    path: &Path,
) -> Result<Arc<DocumentSnapshot>, DocumentRequestError> {
    service
        .documents()
        .open_snapshot(path)
        .ok_or_else(|| DocumentRequestError::DocumentNotOpen {
            path: path.to_path_buf(),
        })
}
