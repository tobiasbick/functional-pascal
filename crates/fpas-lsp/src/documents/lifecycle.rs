//! Workspace loading and ordered editor document mutations.

use std::collections::HashSet;
use std::path::{Path, PathBuf};

use fpas_language_service::{
    CancellationToken, LanguageService, LanguageServiceError, SourceVersion, format_document,
};
use tower_lsp_server::ls_types::{
    DidChangeTextDocumentParams, DidCloseTextDocumentParams, DidOpenTextDocumentParams,
    DidSaveTextDocumentParams, Uri,
};

use super::tasks;
use super::{
    DocumentRequestError, DocumentSyncError, FormattedDocument, SynchronizedDocument,
    SynchronizedDocuments,
};
use crate::convert::file_uri_to_path;

impl SynchronizedDocuments {
    pub(crate) async fn set_workspace(
        &self,
        root: &Path,
        standard_library_root: Option<&Path>,
    ) -> Result<(), LanguageServiceError> {
        let root = root.to_path_buf();
        let error_root = root.clone();
        let standard_library_root = standard_library_root.map(Path::to_path_buf);
        let cancellation = CancellationToken::new();
        let task_cancellation = cancellation.clone();
        let service = tokio::task::spawn_blocking(move || {
            if let Some(standard_library_root) = standard_library_root {
                LanguageService::load_with_standard_library_and_cancellation(
                    &root,
                    &standard_library_root,
                    &task_cancellation,
                )
            } else {
                LanguageService::load_with_cancellation(&root, &task_cancellation)
            }
        })
        .await
        .map_err(|error| LanguageServiceError::Analysis {
            path: error_root,
            message: error.to_string(),
        })??;
        *self.service.lock().await = service;
        Ok(())
    }

    pub(crate) async fn barrier(&self) {
        drop(self.service.lock().await);
    }

    pub(crate) async fn refresh_paths(
        &self,
        paths: Vec<PathBuf>,
    ) -> Result<Vec<SynchronizedDocument>, LanguageServiceError> {
        let mut refreshed = self.service.lock().await.fork_for_queries();
        let cancellation = CancellationToken::new();
        let task_cancellation = cancellation.clone();
        let _cancel_on_drop = tasks::CancelOnDrop(cancellation);
        let (refreshed, documents) = tokio::task::spawn_blocking(move || {
            let mut affected_paths = affected_open_paths(&refreshed, &paths);
            refreshed.refresh_paths(&paths, &task_cancellation)?;
            affected_paths.extend(affected_open_paths(&refreshed, &paths));
            let documents = synchronized_open_documents(&refreshed)
                .into_iter()
                .filter(|document| affected_paths.contains(&document.path))
                .collect();
            Ok::<_, LanguageServiceError>((refreshed, documents))
        })
        .await
        .map_err(|error| LanguageServiceError::Analysis {
            path: PathBuf::new(),
            message: error.to_string(),
        })??;
        *self.service.lock().await = refreshed;
        Ok(documents)
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
        editor_version(snapshot.version())
    }

    pub(crate) async fn analyze_diagnostics_if_current(
        &self,
        path: &Path,
        version: i32,
    ) -> Result<Option<fpas_language_service::DiagnosticAnalysis>, LanguageServiceError> {
        let path = path.to_path_buf();
        tasks::run(&self.service, move |service, cancellation| {
            cancellation.check()?;
            let Some(snapshot) = service.documents().open_snapshot(&path) else {
                return Ok(None);
            };
            if editor_version(snapshot.version()) != Some(version) {
                return Ok(None);
            }
            let analysis = service.analyze_document_diagnostics(&path)?;
            cancellation.check()?;
            Ok(
                (editor_version(analysis.document().snapshot().version()) == Some(version))
                    .then_some(analysis),
            )
        })
        .await
        .map_err(document_request_service_error)
    }

    pub(crate) async fn format_open(
        &self,
        path: &Path,
    ) -> Result<Option<FormattedDocument>, DocumentSyncError> {
        let path = path.to_path_buf();
        tasks::run(&self.service, move |service, cancellation| {
            cancellation.check()?;
            let snapshot = service
                .documents()
                .open_snapshot(&path)
                .ok_or_else(|| DocumentRequestError::DocumentNotOpen { path: path.clone() })?;
            let text = format_document(&snapshot);
            cancellation.check()?;
            Ok(text.map(|text| FormattedDocument { snapshot, text }))
        })
        .await
        .map_err(document_request_sync_error)
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

fn synchronized_open_documents(service: &LanguageService) -> Vec<SynchronizedDocument> {
    service
        .documents()
        .open_snapshots()
        .into_iter()
        .filter_map(|snapshot| {
            Some(SynchronizedDocument {
                path: snapshot.path().to_path_buf(),
                uri: Uri::from_file_path(snapshot.path())?,
                version: editor_version(snapshot.version())?,
            })
        })
        .collect()
}

fn affected_open_paths(service: &LanguageService, changed_paths: &[PathBuf]) -> HashSet<PathBuf> {
    let open_snapshots = service.documents().open_snapshots();
    if service
        .workspace()
        .manifest_path()
        .is_some_and(|manifest| changed_paths.iter().any(|path| path == manifest))
    {
        return open_snapshots
            .into_iter()
            .map(|snapshot| snapshot.path().to_path_buf())
            .collect();
    }

    let changed_project_sources = service
        .workspace()
        .projects()
        .iter()
        .filter(|project| {
            changed_paths
                .iter()
                .any(|path| path == project.manifest_path())
        })
        .flat_map(|project| {
            project
                .source_files()
                .iter()
                .map(PathBuf::as_path)
                .chain(project.main())
        })
        .collect::<Vec<_>>();

    open_snapshots
        .into_iter()
        .filter(|snapshot| {
            changed_paths.iter().any(|path| path == snapshot.path())
                || service.workspace().projects().iter().any(|project| {
                    project.contains_source(snapshot.path())
                        && (changed_paths
                            .iter()
                            .any(|path| project.contains_source(path))
                            || changed_project_sources
                                .iter()
                                .any(|path| project.contains_source(path)))
                })
        })
        .map(|snapshot| snapshot.path().to_path_buf())
        .collect()
}

fn editor_version(version: SourceVersion) -> Option<i32> {
    let SourceVersion::Editor(version) = version else {
        return None;
    };
    i32::try_from(version).ok()
}

fn document_request_service_error(error: DocumentRequestError) -> LanguageServiceError {
    match error {
        DocumentRequestError::Service(error) => error,
        error => LanguageServiceError::Analysis {
            path: PathBuf::new(),
            message: error.to_string(),
        },
    }
}

fn document_request_sync_error(error: DocumentRequestError) -> DocumentSyncError {
    match error {
        DocumentRequestError::Service(error) => DocumentSyncError::Service(error),
        DocumentRequestError::DocumentNotOpen { path } => {
            DocumentSyncError::DocumentNotOpen { path }
        }
        error => DocumentSyncError::Service(LanguageServiceError::Analysis {
            path: PathBuf::new(),
            message: error.to_string(),
        }),
    }
}
