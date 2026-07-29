//! Editor-facing diagnostic accessors.

use fpas_diagnostics::Diagnostic;

use crate::{DocumentAnalysis, DocumentSnapshot};

/// Returns the merged lexer, parser, and semantic diagnostics for an analyzed document.
#[must_use]
pub fn diagnostics_for_document(analysis: &DocumentAnalysis) -> &[Diagnostic] {
    analysis.diagnostics()
}

pub(crate) fn parse_diagnostics(snapshot: &DocumentSnapshot) -> Vec<Diagnostic> {
    snapshot
        .parse_diagnostics()
        .iter()
        .map(|diagnostic| diagnostic.as_diagnostic().clone())
        .collect()
}

pub(crate) fn merged_diagnostics(
    snapshot: &DocumentSnapshot,
    semantic: impl IntoIterator<Item = Diagnostic>,
) -> Vec<Diagnostic> {
    let mut diagnostics = parse_diagnostics(snapshot);
    diagnostics.extend(semantic);
    diagnostics
}
