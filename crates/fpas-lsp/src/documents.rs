//! Full-text LSP synchronization backed by the language service.

use std::fmt;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use fpas_language_service::{
    DocumentAnalysis, DocumentSnapshot, LanguageService, LanguageServiceError, SourceVersion,
    format_document,
};
use tokio::sync::Mutex;
use tower_lsp_server::ls_types::{
    DidChangeTextDocumentParams, DidCloseTextDocumentParams, DidOpenTextDocumentParams,
    DidSaveTextDocumentParams, Uri,
};

use crate::convert::{FileUriError, file_uri_to_path};

/// Synchronized document state shared by concurrent LSP notification handlers.
pub(crate) struct SynchronizedDocuments {
    service: Mutex<LanguageService>,
}

#[derive(Clone)]
pub(crate) struct SynchronizedDocument {
    pub(crate) path: PathBuf,
    pub(crate) uri: Uri,
    pub(crate) version: i32,
}

pub(crate) struct FormattedDocument {
    pub(crate) snapshot: Arc<DocumentSnapshot>,
    pub(crate) text: String,
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
    ) -> Result<SynchronizedDocument, DocumentSyncError> {
        let document = params.text_document;
        let path = file_uri_to_path(&document.uri)?;
        self.service.lock().await.documents_mut().open_document(
            &path,
            i64::from(document.version),
            document.text,
        )?;
        Ok(SynchronizedDocument {
            path,
            uri: document.uri,
            version: document.version,
        })
    }

    pub(crate) fn validate_change(
        params: &DidChangeTextDocumentParams,
    ) -> Result<PathBuf, DocumentSyncError> {
        let path = file_uri_to_path(&params.text_document.uri)?;
        let [change] = params.content_changes.as_slice() else {
            return Err(DocumentSyncError::ExpectedOneFullChange {
                received: params.content_changes.len(),
            });
        };
        if change.range.is_some() {
            return Err(DocumentSyncError::IncrementalChange);
        }
        Ok(path)
    }

    pub(crate) async fn change(
        &self,
        params: DidChangeTextDocumentParams,
    ) -> Result<SynchronizedDocument, DocumentSyncError> {
        let path = Self::validate_change(&params)?;
        let [change] = params.content_changes.as_slice() else {
            unreachable!("validate_change accepted exactly one content change");
        };
        self.service.lock().await.documents_mut().apply_full_text(
            &path,
            i64::from(params.text_document.version),
            change.text.clone(),
        )?;
        Ok(SynchronizedDocument {
            path,
            uri: params.text_document.uri,
            version: params.text_document.version,
        })
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

    pub(crate) async fn open_version(&self, path: &Path) -> Option<i32> {
        let snapshot = self.service.lock().await.documents().open_snapshot(path)?;
        let SourceVersion::Editor(version) = snapshot.version() else {
            return None;
        };
        i32::try_from(version).ok()
    }

    pub(crate) async fn analyze_if_current(
        &self,
        path: &Path,
        version: i32,
    ) -> Result<Option<Arc<DocumentAnalysis>>, LanguageServiceError> {
        let mut service = self.service.lock().await;
        let Some(snapshot) = service.documents().open_snapshot(path) else {
            return Ok(None);
        };
        if snapshot.version() != SourceVersion::Editor(i64::from(version)) {
            return Ok(None);
        }
        let analysis = service.analyze_document(path)?;
        if analysis.snapshot().version() != SourceVersion::Editor(i64::from(version)) {
            return Ok(None);
        }
        Ok(Some(analysis))
    }

    pub(crate) async fn format_open(
        &self,
        path: &Path,
    ) -> Result<Option<FormattedDocument>, DocumentSyncError> {
        let service = self.service.lock().await;
        let snapshot = service.documents().open_snapshot(path).ok_or_else(|| {
            DocumentSyncError::DocumentNotOpen {
                path: path.to_path_buf(),
            }
        })?;
        Ok(format_document(&snapshot).map(|text| FormattedDocument { snapshot, text }))
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
