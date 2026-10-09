//! Attribute project records to their authoritative source or a positionless log message.
//!
//! **Documentation:** `docs/pascal/tools/diagnostics.md`

use fpas_diagnostics::FileDiagnostic;
use fpas_language_service::{DiagnosticAnalysis, SourceVersion};
use tower_lsp_server::ls_types::Uri;

use super::convert::file_diagnostic_to_lsp;
use super::publication::DiagnosticBatch;

/// Separates project records with convertible source ranges from positionless failures.
pub(super) fn project_diagnostics(
    analysis: &DiagnosticAnalysis,
) -> (Vec<DiagnosticBatch>, Vec<FileDiagnostic>) {
    let mut batches = Vec::new();
    let mut unlocated = Vec::new();
    let Some(failure) = analysis.failure() else {
        return (batches, unlocated);
    };
    for record in failure.diagnostics() {
        let located = analysis.failure_snapshot().and_then(|snapshot| {
            if record.path.as_deref() != Some(snapshot.path()) {
                return None;
            }
            let uri = Uri::from_file_path(snapshot.path())?;
            let diagnostic = file_diagnostic_to_lsp(snapshot, &record).ok()?;
            let version = match snapshot.version() {
                SourceVersion::Editor(version) => i32::try_from(version).ok(),
                SourceVersion::Disk(_) => None,
            };
            Some(DiagnosticBatch {
                uri,
                version,
                revision: snapshot.revision(),
                diagnostics: vec![diagnostic],
            })
        });
        match located {
            Some(batch) => batches.push(batch),
            None => unlocated.push(record),
        }
    }
    (batches, unlocated)
}
