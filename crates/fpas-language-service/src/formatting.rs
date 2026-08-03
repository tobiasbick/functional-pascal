//! Canonical formatting query for immutable source snapshots.

use crate::DocumentSnapshot;

/// Formats an unsaved snapshot through `fpas-fmt`.
///
/// Returns `None` when parse errors or an inconsistent source/AST snapshot make canonical
/// formatting unsafe.
#[must_use]
pub fn format_document(snapshot: &DocumentSnapshot) -> Option<String> {
    if snapshot.has_parse_errors() {
        return None;
    }
    fpas_fmt::format_source(snapshot.source(), snapshot.compilation_unit()).ok()
}
