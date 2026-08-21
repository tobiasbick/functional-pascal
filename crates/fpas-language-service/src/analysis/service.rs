//! Stateful source service that keeps editor overlays and analysis caches independent from LSP.

use std::path::Path;
use std::sync::Arc;

use fpas_parser::CompilationUnit;

use super::cache::{AnalysisCache, AnalysisFingerprint, AnalysisSet};
use super::document::{DiagnosticAnalysis, DocumentAnalysis, semantic_document};
use super::project::{analyze_project, project_identity};
use crate::workspace::StandardLibraryContext;
use crate::{
    DocumentSnapshot, DocumentStore, DocumentSymbols, LanguageServiceError, ProjectContext,
    WorkspaceContext, WorkspaceSymbolIndex,
};

/// Stateful source service that keeps editor overlays and analysis caches independent from LSP.
pub struct LanguageService {
    documents: DocumentStore,
    workspace: WorkspaceContext,
    standard_library: Option<StandardLibraryContext>,
    analysis_cache: AnalysisCache,
}

impl LanguageService {
    pub(crate) fn editor_api_source_paths(&self) -> Vec<std::path::PathBuf> {
        self.standard_library
            .as_ref()
            .map(|library| library.editor_api_sources().to_vec())
            .unwrap_or_default()
    }

    pub(crate) fn is_editor_api_source(&self, path: &Path) -> bool {
        let path = crate::document::normalized_path(path);
        self.standard_library.as_ref().is_some_and(|library| {
            library
                .editor_api_sources()
                .iter()
                .any(|source| source == &path)
        })
    }

    /// Creates a service for an already loaded workspace context.
    #[must_use]
    pub fn new(workspace: WorkspaceContext) -> Self {
        Self {
            documents: DocumentStore::new(),
            workspace,
            standard_library: None,
            analysis_cache: AnalysisCache::default(),
        }
    }

    /// Creates an isolated query service over the current immutable source snapshots.
    ///
    /// The returned service shares parsed snapshots but owns an independent document map and
    /// analysis cache. Later editor mutations on either service do not affect the other.
    #[must_use]
    pub fn fork_for_queries(&self) -> Self {
        Self {
            documents: self.documents.clone(),
            workspace: self.workspace.clone(),
            standard_library: self.standard_library.clone(),
            analysis_cache: AnalysisCache::default(),
        }
    }

    /// Discovers and loads source, project, or workspace context.
    #[must_use]
    pub fn load(input: &Path) -> Self {
        Self::new(WorkspaceContext::load(input))
    }

    /// Discovers editor context while observing a cooperative cancellation signal.
    pub fn load_with_cancellation(
        input: &Path,
        cancellation: &crate::CancellationToken,
    ) -> Result<Self, LanguageServiceError> {
        Ok(Self::new(WorkspaceContext::load_with_cancellation(
            input,
            cancellation,
        )?))
    }

    /// Loads editor context together with an implementation-owned source standard library.
    ///
    /// # Errors
    ///
    /// Returns an analysis error when the standard-library root or manifest is invalid.
    pub fn load_with_standard_library(
        input: &Path,
        standard_library_root: &Path,
    ) -> Result<Self, LanguageServiceError> {
        Self::load_with_standard_library_and_cancellation(
            input,
            standard_library_root,
            &crate::CancellationToken::new(),
        )
    }

    /// Loads editor context and the source standard library with cooperative cancellation.
    pub fn load_with_standard_library_and_cancellation(
        input: &Path,
        standard_library_root: &Path,
        cancellation: &crate::CancellationToken,
    ) -> Result<Self, LanguageServiceError> {
        cancellation.check()?;
        let standard_library = StandardLibraryContext::load(standard_library_root)
            .map_err(|message| LanguageServiceError::analysis(standard_library_root, message))?;
        cancellation.check()?;
        Ok(Self {
            standard_library: Some(standard_library),
            ..Self::load_with_cancellation(input, cancellation)?
        })
    }

    /// Returns the recoverable project/workspace context.
    #[must_use]
    pub fn workspace(&self) -> &WorkspaceContext {
        &self.workspace
    }

    /// Returns the versioned document store.
    #[must_use]
    pub fn documents(&self) -> &DocumentStore {
        &self.documents
    }

    /// Returns mutable access for full-text open/change/close synchronization.
    pub fn documents_mut(&mut self) -> &mut DocumentStore {
        &mut self.documents
    }

    /// Loads the authoritative open-buffer or disk snapshot for a source path.
    pub fn snapshot(&mut self, path: &Path) -> Result<Arc<DocumentSnapshot>, LanguageServiceError> {
        self.documents.snapshot(path)
    }

    /// Refreshes changed sources and the bounded folder catalog while preserving open buffers.
    pub fn refresh_paths(
        &mut self,
        paths: &[std::path::PathBuf],
        cancellation: &crate::CancellationToken,
    ) -> Result<(), LanguageServiceError> {
        cancellation.check()?;
        for path in paths {
            self.documents.invalidate_disk(path);
            self.analysis_cache.invalidate_path(path);
        }
        let changed_projects = self.workspace.reload_folder(cancellation)?;
        self.analysis_cache.invalidate_identities(&changed_projects);
        cancellation.check()
    }

    /// Returns cached or newly computed project-aware analysis for one document.
    pub fn analyze_document(
        &mut self,
        path: &Path,
    ) -> Result<Arc<DocumentAnalysis>, LanguageServiceError> {
        self.ensure_source_context(path)?;
        let target = self.documents.snapshot(path)?;
        if self.is_editor_api_source(path) {
            return self.cached_syntax_only(target);
        }
        if target.has_parse_errors() {
            return self.cached_syntax_only(target);
        }

        let project = self.analysis_project_for(path, target.compilation_unit());
        let Some(project) = project else {
            return self.analyze_loose(target);
        };
        let snapshots = self.project_snapshots(&project)?;
        let fingerprint = AnalysisFingerprint::new(project_identity(&project), &snapshots);
        let set = if let Some(cached) = self.analysis_cache.get(&fingerprint) {
            cached
        } else {
            let analysis = analyze_project(&project, &snapshots)?;
            self.analysis_cache.insert(fingerprint, analysis)
        };
        set.document(path).ok_or_else(|| {
            LanguageServiceError::analysis(
                path,
                "The source is not present in its loaded project analysis.",
            )
        })
    }

    /// Returns current diagnostics even when project-wide analysis cannot read a sibling source.
    pub fn analyze_document_diagnostics(
        &mut self,
        path: &Path,
    ) -> Result<DiagnosticAnalysis, LanguageServiceError> {
        let target = self.documents.snapshot(path)?;
        let result = (|| {
            self.ensure_source_context(path)?;
            if !self.is_editor_api_source(path)
                && let Some(project) = self.analysis_project_for(path, target.compilation_unit())
            {
                self.project_snapshots(&project)?;
            }
            self.analyze_document(path)
        })();
        match result {
            Ok(document) => Ok(DiagnosticAnalysis::from_outcome(document, None)),
            Err(failure) => Ok(DiagnosticAnalysis::from_outcome(
                self.cached_syntax_only(target)?,
                Some(failure),
            )),
        }
    }

    /// Builds a collision-safe symbol index for every source in the current project catalog.
    pub fn workspace_symbol_index(&mut self) -> Result<WorkspaceSymbolIndex, LanguageServiceError> {
        let mut index = WorkspaceSymbolIndex::new();
        let mut paths = self
            .workspace
            .projects()
            .iter()
            .map(|project| {
                self.standard_library
                    .as_ref()
                    .map_or_else(|| project.clone(), |library| library.compose(project))
            })
            .flat_map(|project| project.all_source_paths())
            .collect::<Vec<_>>();
        paths.extend(
            self.documents
                .open_snapshots()
                .into_iter()
                .map(|snapshot| snapshot.path().to_path_buf()),
        );
        if let Some(standard_library) = &self.standard_library {
            paths.extend(standard_library.editor_api_sources().iter().cloned());
        }
        paths.sort();
        paths.dedup();
        let documents = paths
            .into_iter()
            .map(|path| {
                let snapshot = self.snapshot(&path)?;
                let symbols = if self.is_editor_api_source(&path) {
                    DocumentSymbols::from_editor_snapshot(&snapshot)
                } else {
                    DocumentSymbols::from_snapshot(&snapshot)
                };
                Ok((path, symbols))
            })
            .collect::<Result<Vec<_>, LanguageServiceError>>()?;
        index.replace_documents(documents);
        Ok(index)
    }

    fn project_snapshots(
        &mut self,
        project: &crate::ProjectContext,
    ) -> Result<Vec<Arc<DocumentSnapshot>>, LanguageServiceError> {
        project
            .all_source_paths()
            .iter()
            .map(|path| self.documents.snapshot(path))
            .collect()
    }

    pub(crate) fn ensure_source_context(
        &mut self,
        path: &Path,
    ) -> Result<(), LanguageServiceError> {
        if self.is_editor_api_source(path) {
            return Ok(());
        }
        self.workspace
            .discover_project_for_source(path)
            .map_err(|issue| {
                LanguageServiceError::analysis(
                    path,
                    format!(
                        "Cannot resolve the FPAS project from `{}`: {}",
                        issue.path.display(),
                        issue.message
                    ),
                )
            })
    }

    pub(crate) fn analysis_project_for(
        &self,
        path: &Path,
        compilation_unit: &CompilationUnit,
    ) -> Option<ProjectContext> {
        if let Some(project) = self.workspace.project_for_source(path) {
            return Some(
                self.standard_library
                    .as_ref()
                    .map_or_else(|| project.clone(), |library| library.compose(project)),
            );
        }
        let standard_library = self.standard_library.as_ref()?;
        let kind = match compilation_unit {
            CompilationUnit::Program(_) => fpas_project::ProjectKind::Program,
            CompilationUnit::Unit(_) => fpas_project::ProjectKind::Library,
        };
        Some(standard_library.compose_loose(path, kind))
    }

    fn cached_syntax_only(
        &mut self,
        snapshot: Arc<DocumentSnapshot>,
    ) -> Result<Arc<DocumentAnalysis>, LanguageServiceError> {
        let fingerprint = AnalysisFingerprint::new(snapshot.path(), &[Arc::clone(&snapshot)]);
        let set = if let Some(cached) = self.analysis_cache.get(&fingerprint) {
            cached
        } else {
            let analysis = Arc::new(DocumentAnalysis::syntax_only(Arc::clone(&snapshot)));
            self.analysis_cache
                .insert(fingerprint, AnalysisSet::one(analysis))
        };
        set.document(snapshot.path()).ok_or_else(|| {
            LanguageServiceError::analysis(
                snapshot.path(),
                "Cached syntax analysis lost its document.",
            )
        })
    }

    fn analyze_loose(
        &mut self,
        snapshot: Arc<DocumentSnapshot>,
    ) -> Result<Arc<DocumentAnalysis>, LanguageServiceError> {
        let fingerprint = AnalysisFingerprint::new(snapshot.path(), &[Arc::clone(&snapshot)]);
        let set = if let Some(cached) = self.analysis_cache.get(&fingerprint) {
            cached
        } else {
            let metadata = match snapshot.compilation_unit() {
                CompilationUnit::Program(program) => {
                    fpas_sema::analyze_program_with_interfaces(program, &[])
                }
                CompilationUnit::Unit(unit) => {
                    fpas_sema::analyze_unit(unit, &[]).map(|analysis| analysis.metadata)
                }
            }
            .map_err(|error| LanguageServiceError::analysis(snapshot.path(), error.to_string()))?;
            let analysis = Arc::new(semantic_document(Arc::clone(&snapshot), metadata));
            self.analysis_cache
                .insert(fingerprint, AnalysisSet::one(analysis))
        };
        set.document(snapshot.path()).ok_or_else(|| {
            LanguageServiceError::analysis(
                snapshot.path(),
                "Cached loose-file analysis lost its document.",
            )
        })
    }
}
