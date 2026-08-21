//! Parse, semantic, and diagnostic results for one source snapshot.

use std::sync::Arc;

use fpas_diagnostics::Diagnostic;
use fpas_sema::AnalysisMetadata;

use crate::diagnostics::{merged_diagnostics, parse_diagnostics};
use crate::{DocumentSnapshot, DocumentSymbols, LanguageServiceError};

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

/// Current document diagnostics plus an optional project-analysis failure.
pub struct DiagnosticAnalysis {
    document: Arc<DocumentAnalysis>,
    failure: Option<LanguageServiceError>,
}

impl DiagnosticAnalysis {
    pub(super) fn from_outcome(
        document: Arc<DocumentAnalysis>,
        failure: Option<LanguageServiceError>,
    ) -> Self {
        Self { document, failure }
    }

    /// Returns syntax or full semantic diagnostics for the current snapshot.
    #[must_use]
    pub fn document(&self) -> &Arc<DocumentAnalysis> {
        &self.document
    }

    /// Returns the project failure that prevented complete semantic diagnostics.
    #[must_use]
    pub fn failure(&self) -> Option<&LanguageServiceError> {
        self.failure.as_ref()
    }
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

pub(crate) fn semantic_document(
    snapshot: Arc<DocumentSnapshot>,
    metadata: AnalysisMetadata,
) -> DocumentAnalysis {
    let diagnostics = merged_diagnostics(&snapshot, metadata.errors.iter().cloned()).into();
    let symbols = DocumentSymbols::from_snapshot(&snapshot);
    DocumentAnalysis {
        snapshot,
        diagnostics,
        semantic: Some(Arc::new(SemanticAnalysis { metadata })),
        symbols,
    }
}
