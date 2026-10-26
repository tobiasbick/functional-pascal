//! Conservative, snapshot-bound source refactorings.

mod extract;
mod imports;

use std::ops::Range;
use std::path::Path;

use crate::{LanguageService, LanguageServiceError, NavigationResult, SemanticEdit, SourceVersion};

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
            && let Some(edits) = imports::organize(&context.documents, index)
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
        &self,
        snapshot: &crate::DocumentSnapshot,
        edits: &[SemanticEdit],
    ) -> Result<bool, LanguageServiceError> {
        let Some(source) = apply_edits(snapshot.source(), edits) else {
            return Ok(false);
        };
        let version = match snapshot.version() {
            SourceVersion::Editor(version) => version.checked_add(1),
            SourceVersion::Disk(_) => Some(1),
        };
        let Some(version) = version else {
            return Ok(false);
        };
        let mut candidate = self.fork_for_edits();
        candidate
            .documents_mut()
            .open_document(snapshot.path(), version, source)?;
        let analysis = candidate.analyze_document(snapshot.path())?;
        Ok(analysis.semantic().is_some()
            && !analysis
                .diagnostics()
                .iter()
                .any(|diagnostic| diagnostic.is_error()))
    }
}

fn apply_edits(source: &str, edits: &[SemanticEdit]) -> Option<String> {
    let mut result = source.to_owned();
    let mut ordered = edits.iter().collect::<Vec<_>>();
    ordered.sort_by_key(|edit| std::cmp::Reverse(edit.span.offset()));
    let mut previous = source.len();
    for edit in ordered {
        if edit.span.end() > previous || source.get(edit.span.offset()..edit.span.end()).is_none() {
            return None;
        }
        result.replace_range(edit.span.offset()..edit.span.end(), &edit.new_text);
        previous = edit.span.offset();
    }
    Some(result)
}
