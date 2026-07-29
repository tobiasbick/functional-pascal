//! Full-text LSP synchronization backed by the language service.

use std::fmt;
use std::path::{Path, PathBuf};

use fpas_language_service::{LanguageService, LanguageServiceError};
use tokio::sync::Mutex;
use tower_lsp_server::ls_types::{
    DidChangeTextDocumentParams, DidCloseTextDocumentParams, DidOpenTextDocumentParams,
    DidSaveTextDocumentParams,
};

use crate::convert::{FileUriError, file_uri_to_path};

/// Synchronized document state shared by concurrent LSP notification handlers.
pub(crate) struct SynchronizedDocuments {
    service: Mutex<LanguageService>,
}

impl SynchronizedDocuments {
    pub(crate) fn new(initial_root: PathBuf) -> Self {
        Self {
            service: Mutex::new(LanguageService::load(&initial_root)),
        }
    }

    pub(crate) async fn set_workspace_root(&self, root: &Path) {
        *self.service.lock().await = LanguageService::load(root);
    }

    pub(crate) async fn barrier(&self) {
        drop(self.service.lock().await);
    }

    pub(crate) async fn open(
        &self,
        params: DidOpenTextDocumentParams,
    ) -> Result<(), DocumentSyncError> {
        let document = params.text_document;
        let path = file_uri_to_path(&document.uri)?;
        self.service.lock().await.documents_mut().open_document(
            &path,
            i64::from(document.version),
            document.text,
        )?;
        Ok(())
    }

    pub(crate) async fn change(
        &self,
        params: DidChangeTextDocumentParams,
    ) -> Result<(), DocumentSyncError> {
        let path = file_uri_to_path(&params.text_document.uri)?;
        let [change] = params.content_changes.as_slice() else {
            return Err(DocumentSyncError::ExpectedOneFullChange {
                received: params.content_changes.len(),
            });
        };
        if change.range.is_some() {
            return Err(DocumentSyncError::IncrementalChange);
        }
        self.service.lock().await.documents_mut().apply_full_text(
            &path,
            i64::from(params.text_document.version),
            change.text.clone(),
        )?;
        Ok(())
    }

    pub(crate) async fn save(
        &self,
        params: DidSaveTextDocumentParams,
    ) -> Result<(), DocumentSyncError> {
        let path = file_uri_to_path(&params.text_document.uri)?;
        if !self.service.lock().await.documents().is_open(&path) {
            return Err(DocumentSyncError::DocumentNotOpen { path });
        }
        Ok(())
    }

    pub(crate) async fn close(
        &self,
        params: DidCloseTextDocumentParams,
    ) -> Result<(), DocumentSyncError> {
        let path = file_uri_to_path(&params.text_document.uri)?;
        self.service
            .lock()
            .await
            .documents_mut()
            .close_document(&path);
        Ok(())
    }
}

/// A recoverable invalid document synchronization notification.
#[derive(Debug)]
pub(crate) enum DocumentSyncError {
    Uri(FileUriError),
    Service(LanguageServiceError),
    ExpectedOneFullChange { received: usize },
    IncrementalChange,
    DocumentNotOpen { path: PathBuf },
}

impl fmt::Display for DocumentSyncError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Uri(error) => error.fmt(formatter),
            Self::Service(error) => error.fmt(formatter),
            Self::ExpectedOneFullChange { received } => write!(
                formatter,
                "Expected exactly one full-document content change, received {received}."
            ),
            Self::IncrementalChange => write!(
                formatter,
                "Incremental text changes are unsupported; send one full-document change."
            ),
            Self::DocumentNotOpen { path } => write!(
                formatter,
                "Cannot save `{}` because the document is not open.",
                path.display()
            ),
        }
    }
}

impl From<FileUriError> for DocumentSyncError {
    fn from(error: FileUriError) -> Self {
        Self::Uri(error)
    }
}

impl From<LanguageServiceError> for DocumentSyncError {
    fn from(error: LanguageServiceError) -> Self {
        Self::Service(error)
    }
}
