//! Full-text LSP synchronization backed by the language service.

use std::fmt;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use fpas_language_service::{
    CancellationToken, DocumentAnalysis, DocumentSnapshot, DocumentSymbol, HoverInfo,
    LanguageService, LanguageServiceError, NavigationResult, ReferenceLocation, RenameEdit,
    RenameError, RenameTarget, SourceVersion, SymbolLocation, format_document,
};
use tokio::sync::Mutex;
use tower_lsp_server::ls_types::{
    DidChangeTextDocumentParams, DidCloseTextDocumentParams, DidOpenTextDocumentParams,
    DidSaveTextDocumentParams, Position, Uri,
};

use crate::convert::{
    FileUriError, PositionConversionError, file_uri_to_path, position_to_byte_offset,
};

/// Synchronized document state shared by concurrent LSP notification handlers.
pub(crate) struct SynchronizedDocuments {
    pub(crate) service: Arc<Mutex<LanguageService>>,
}

struct CancelOnDrop(CancellationToken);

impl Drop for CancelOnDrop {
    fn drop(&mut self) {
        self.0.cancel();
    }
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

pub(crate) struct DefinitionDocument {
    pub(crate) snapshot: Arc<DocumentSnapshot>,
    pub(crate) location: SymbolLocation,
}

pub(crate) struct ReferenceDocument {
    pub(crate) snapshot: Arc<DocumentSnapshot>,
    pub(crate) location: ReferenceLocation,
}

pub(crate) struct RenameDocument {
    pub(crate) snapshot: Arc<DocumentSnapshot>,
    pub(crate) edit: RenameEdit,
}

impl SynchronizedDocuments {
    pub(crate) fn new(initial_root: PathBuf) -> Self {
        Self {
            service: Arc::new(Mutex::new(LanguageService::load(&initial_root))),
        }
    }

    pub(crate) async fn set_workspace(
        &self,
        root: &Path,
        standard_library_root: Option<&Path>,
    ) -> Result<(), LanguageServiceError> {
        let shared_service = Arc::clone(&self.service);
        let root = root.to_path_buf();
        let error_root = root.clone();
        let standard_library_root = standard_library_root.map(Path::to_path_buf);
        let cancellation = CancellationToken::new();
        let task_cancellation = cancellation.clone();
        let _cancel_on_drop = CancelOnDrop(cancellation);
        tokio::task::spawn_blocking(move || {
            let service = if let Some(standard_library_root) = standard_library_root {
                LanguageService::load_with_standard_library_and_cancellation(
                    &root,
                    &standard_library_root,
                    &task_cancellation,
                )?
            } else {
                LanguageService::load_with_cancellation(&root, &task_cancellation)?
            };
            *shared_service.blocking_lock() = service;
            Ok(())
        })
        .await
        .map_err(|error| LanguageServiceError::Analysis {
            path: error_root,
            message: error.to_string(),
        })?
    }

    pub(crate) async fn barrier(&self) {
        drop(self.service.lock().await);
    }

    pub(crate) async fn refresh_paths(
        &self,
        paths: Vec<PathBuf>,
    ) -> Result<Vec<SynchronizedDocument>, LanguageServiceError> {
        let mut service = self.service.lock().await;
        service.refresh_paths(&paths, &CancellationToken::new())?;
        Ok(service
            .documents()
            .open_snapshots()
            .into_iter()
            .filter_map(|snapshot| {
                let SourceVersion::Editor(version) = snapshot.version() else {
                    return None;
                };
                Some(SynchronizedDocument {
                    path: snapshot.path().to_path_buf(),
                    uri: Uri::from_file_path(snapshot.path())?,
                    version: i32::try_from(version).ok()?,
                })
            })
            .collect())
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

    pub(crate) async fn document_symbols_open(
        &self,
        path: &Path,
    ) -> Result<NavigationResult<Vec<DocumentSymbol>>, DocumentRequestError> {
        let mut service = self.service.lock().await;
        require_open(&service, path)?;
        Ok(service.document_symbols(path)?)
    }

    pub(crate) async fn hover_open(
        &self,
        path: &Path,
        position: Position,
    ) -> Result<NavigationResult<Option<HoverInfo>>, DocumentRequestError> {
        let mut service = self.service.lock().await;
        let snapshot = require_open(&service, path)?;
        let offset = position_to_byte_offset(&snapshot, position)?;
        Ok(service.hover(path, offset)?)
    }

    pub(crate) async fn definitions_open(
        &self,
        path: &Path,
        position: Position,
    ) -> Result<Vec<DefinitionDocument>, DocumentRequestError> {
        let mut service = self.service.lock().await;
        let snapshot = require_open(&service, path)?;
        let offset = position_to_byte_offset(&snapshot, position)?;
        let result = service.definitions(path, offset)?;
        let mut definitions = Vec::with_capacity(result.value.len());
        for location in result.value {
            definitions.push(DefinitionDocument {
                snapshot: service.snapshot(&location.path)?,
                location,
            });
        }
        Ok(definitions)
    }

    pub(crate) async fn references_open(
        &self,
        path: &Path,
        position: Position,
        include_declaration: bool,
    ) -> Result<Vec<ReferenceDocument>, DocumentRequestError> {
        crate::request_tasks::references(
            Arc::clone(&self.service),
            path.to_path_buf(),
            position,
            include_declaration,
        )
        .await
    }

    pub(crate) async fn prepare_rename_open(
        &self,
        path: &Path,
        position: Position,
    ) -> Result<NavigationResult<Option<RenameTarget>>, DocumentRequestError> {
        let mut service = self.service.lock().await;
        let snapshot = require_open(&service, path)?;
        let offset = position_to_byte_offset(&snapshot, position)?;
        Ok(service.prepare_rename(path, offset)?)
    }

    pub(crate) async fn rename_open(
        &self,
        path: &Path,
        position: Position,
        new_name: &str,
    ) -> Result<Vec<RenameDocument>, DocumentRequestError> {
        crate::request_tasks::rename(
            Arc::clone(&self.service),
            path.to_path_buf(),
            position,
            new_name.to_owned(),
        )
        .await
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

#[derive(Debug)]
pub(crate) enum DocumentRequestError {
    Service(LanguageServiceError),
    Rename(RenameError),
    Position(PositionConversionError),
    DocumentNotOpen { path: PathBuf },
    Task(String),
}

impl fmt::Display for DocumentRequestError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Service(error) => error.fmt(formatter),
            Self::Rename(error) => error.fmt(formatter),
            Self::Position(error) => error.fmt(formatter),
            Self::DocumentNotOpen { path } => write!(
                formatter,
                "Cannot query `{}` because the document is not open.",
                path.display()
            ),
            Self::Task(message) => write!(formatter, "Language-service task failed: {message}"),
        }
    }
}

impl From<LanguageServiceError> for DocumentRequestError {
    fn from(error: LanguageServiceError) -> Self {
        Self::Service(error)
    }
}

impl From<RenameError> for DocumentRequestError {
    fn from(error: RenameError) -> Self {
        Self::Rename(error)
    }
}

impl From<PositionConversionError> for DocumentRequestError {
    fn from(error: PositionConversionError) -> Self {
        Self::Position(error)
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
