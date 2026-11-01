//! Conservative, snapshot-bound source refactorings.

mod add_parameter;
mod edit;
mod extract;
mod imports;
mod plan;
mod prepare;
mod recipe;
mod revalidate;
mod validation;
pub use edit::{CheckedChanges, SourceChange, SourceEdit};
pub use plan::{PlannedEdit, REFACTOR_JSON_BYTES, RefactorPlan};
pub use recipe::{
    ParameterMode, RefactorBlocker, RefactorPreparation, RefactorRecipe, RefactorStatus,
    SourceSelector,
};

use std::ops::Range;
use std::path::Path;

use crate::{
    CancellationToken, LanguageService, LanguageServiceError, NavigationResult, SemanticEdit,
};

/// User-facing refactoring category, independent from an editor protocol.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RefactoringKind {
    /// Replace one expression with a new immutable local binding.
    ExtractVariable,
    /// Sort imports and remove unused imports whose initialization is inert.
    OrganizeImports,
}

/// One complete source transformation grounded in an analyzed snapshot.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Refactoring {
    /// Editor action title.
    pub title: String,
    /// Standard action family.
    pub kind: RefactoringKind,
    /// Non-overlapping edits against the returned source snapshot.
    pub edits: Vec<SemanticEdit>,
}

impl LanguageService {
    /// Offers checked refactorings for an exact UTF-8 selection in an open source.
    ///
    /// Parse/type errors, generated APIs, and sources outside the editor root receive no edits.
    pub fn refactorings(
        &mut self,
        path: &Path,
        selection: Range<usize>,
    ) -> Result<NavigationResult<Vec<Refactoring>>, LanguageServiceError> {
        let analysis = self.analyze_document(path)?;
        let snapshot = analysis.snapshot().clone();
        let mut value = Vec::new();
        if !crate::workspace::path_containment::contains(self.workspace().root(), snapshot.path())
            || self.is_editor_api_source(path)
            || analysis.semantic().is_none()
            || analysis
                .diagnostics()
                .iter()
                .any(|diagnostic| diagnostic.is_error())
            || selection.start > selection.end
            || snapshot.source().get(selection.clone()).is_none()
        {
            return Ok(NavigationResult { snapshot, value });
        }
        let context = self.navigation_context(path)?;
        if context.snapshot.source() != snapshot.source() {
            return Ok(NavigationResult { snapshot, value });
        }
        if let Some(index) = context.target_index
            && let Some(edits) = extract::extract_variable(&context.documents, index, selection)
            && self.valid_refactoring(&snapshot, &edits)?
        {
            value.push(Refactoring {
                title: "Extract variable".into(),
                kind: RefactoringKind::ExtractVariable,
                edits,
            });
        }
        if let Some(index) = context.target_index
            && let Ok(edits) = imports::organize(&context.documents, index)
            && !edits.is_empty()
            && self.valid_refactoring(&snapshot, &edits)?
        {
            value.push(Refactoring {
                title: "Organize imports".into(),
                kind: RefactoringKind::OrganizeImports,
                edits,
            });
        }
        Ok(NavigationResult { snapshot, value })
    }

    fn valid_refactoring(
        &mut self,
        snapshot: &std::sync::Arc<crate::DocumentSnapshot>,
        edits: &[SemanticEdit],
    ) -> Result<bool, LanguageServiceError> {
        let snapshot = std::sync::Arc::clone(snapshot);
        let edits = edits
            .iter()
            .map(|edit| SourceEdit {
                path: snapshot.path().to_path_buf(),
                range: edit.span,
                new_text: edit.new_text.clone(),
                snapshot: std::sync::Arc::clone(&snapshot),
            })
            .collect::<Vec<_>>();
        Ok(self
            .validate_source_edits(snapshot.path(), &edits, &CancellationToken::new())
            .is_ok())
    }
}
