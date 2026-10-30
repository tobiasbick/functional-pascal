//! Editor overlays replace stale coverage diagnostics after explicit finite handling.

use crate::support;

use fpas_diagnostics::codes::{SEMA_CASE_ELSE_FORBIDDEN, SEMA_NON_EXHAUSTIVE_CASE};
use fpas_language_service::{LanguageService, WorkspaceContext};
use support::TempDirectory;

#[test]
fn editing_finite_cases_replaces_coverage_diagnostics_without_changing_disk()
-> Result<(), Box<dyn std::error::Error>> {
    let temp = TempDirectory::new("case-coverage-overlay");
    let disk = "program T; type Digit = 1..3;
        begin case Digit(1) of when 1..2: null; end case; end.";
    let path = temp.write("case.fpas", disk);
    let mut service = LanguageService::new(WorkspaceContext::loose(temp.path()));
    let initial = service.analyze_document(&path)?;
    assert!(
        initial
            .diagnostics()
            .iter()
            .any(|error| error.code == SEMA_NON_EXHAUSTIVE_CASE
                && error.message.contains("missing 3")),
        "{:?}",
        initial.diagnostics()
    );

    service.documents_mut().open_document(
        &path,
        1,
        "program T; begin case true of when true: null; else null; end case; end.",
    )?;
    let invalid = service.analyze_document(&path)?;
    assert!(
        invalid
            .diagnostics()
            .iter()
            .any(|error| error.code == SEMA_CASE_ELSE_FORBIDDEN)
    );
    assert!(
        invalid
            .diagnostics()
            .iter()
            .all(|error| error.code != SEMA_NON_EXHAUSTIVE_CASE)
    );

    service.documents_mut().apply_full_text(
        &path,
        2,
        "program T; begin case true of when true: null; when false: null; end case; end.",
    )?;
    let repaired = service.analyze_document(&path)?;
    assert!(
        repaired.diagnostics().is_empty(),
        "{:?}",
        repaired.diagnostics()
    );
    assert_eq!(std::fs::read_to_string(&path)?, disk);
    Ok(())
}
