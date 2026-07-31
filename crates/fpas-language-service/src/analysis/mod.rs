//! Cached document and project semantic analysis.

mod cache;
mod project;

use std::path::Path;
use std::sync::Arc;

use cache::{AnalysisCache, AnalysisFingerprint, AnalysisSet};
use fpas_diagnostics::Diagnostic;
use fpas_parser::CompilationUnit;
use fpas_sema::AnalysisMetadata;
use project::{analyze_project, project_identity};

use crate::diagnostics::{merged_diagnostics, parse_diagnostics};
use crate::workspace::StandardLibraryContext;
use crate::{
    DocumentSnapshot, DocumentStore, DocumentSymbols, LanguageServiceError, ProjectContext,
    WorkspaceContext, WorkspaceSymbolIndex,
};

/// Compiler semantic metadata tied to the immutable AST allocation in a document snapshot.
pub struct SemanticAnalysis {
    metadata: AnalysisMetadata,
}

impl SemanticAnalysis {
    /// Returns compiler expression types and lowering metadata for the snapshot AST.
    #[must_use]
    pub fn metadata(&self) -> &AnalysisMetadata {
        &self.metadata
    }
}

/// Immutable parse, semantic, diagnostic, and declaration results for one source version.
pub struct DocumentAnalysis {
    snapshot: Arc<DocumentSnapshot>,
    diagnostics: Arc<[Diagnostic]>,
    semantic: Option<Arc<SemanticAnalysis>>,
    symbols: DocumentSymbols,
}

impl DocumentAnalysis {
    pub(super) fn syntax_only(snapshot: Arc<DocumentSnapshot>) -> Self {
        let diagnostics = parse_diagnostics(&snapshot).into();
        let symbols = DocumentSymbols::from_snapshot(&snapshot);
        Self {
            snapshot,
            diagnostics,
            semantic: None,
            symbols,
        }
    }

    /// Returns the exact parsed snapshot analyzed by this result.
    #[must_use]
    pub fn snapshot(&self) -> &Arc<DocumentSnapshot> {
        &self.snapshot
    }

    /// Returns merged lexer, parser, and semantic diagnostics.
    #[must_use]
    pub fn diagnostics(&self) -> &[Diagnostic] {
        &self.diagnostics
    }

    /// Returns semantic metadata when parsing permitted analysis.
    #[must_use]
    pub fn semantic(&self) -> Option<&SemanticAnalysis> {
        self.semantic.as_deref()
    }

    /// Returns declaration symbols for the recovered AST.
    #[must_use]
    pub fn symbols(&self) -> &DocumentSymbols {
        &self.symbols
    }
}

pub(super) fn semantic_document(
    snapshot: Arc<DocumentSnapshot>,
    metadata: AnalysisMetadata,
) -> DocumentAnalysis {
    let diagnostics = merged_diagnostics(&snapshot, metadata.0.iter().cloned()).into();
    let symbols = DocumentSymbols::from_snapshot(&snapshot);
    DocumentAnalysis {
        snapshot,
        diagnostics,
        semantic: Some(Arc::new(SemanticAnalysis { metadata })),
        symbols,
    }
}

/// Stateful source service that keeps editor overlays and analysis caches independent from LSP.
pub struct LanguageService {
    documents: DocumentStore,
    workspace: WorkspaceContext,
    standard_library: Option<StandardLibraryContext>,
    analysis_cache: AnalysisCache,
}

impl LanguageService {
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

    /// Builds a collision-safe symbol index for every analyzable source in the current context.
    pub fn workspace_symbol_index(&mut self) -> Result<WorkspaceSymbolIndex, LanguageServiceError> {
        let mut index = WorkspaceSymbolIndex::new();
        if self.workspace.projects().is_empty() {
            return Ok(index);
        }
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
        paths.sort();
        paths.dedup();
        for path in paths {
            let analysis = self.analyze_document(&path)?;
            index.replace_document(&path, analysis.symbols().clone());
        }
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
