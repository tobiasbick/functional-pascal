//! Editor overlays publish current enum diagnostics and clear them after repair.

mod support;

use fpas_diagnostics::codes::{SEMA_BACKING_VALUE_IN_DATA_ENUM, SEMA_DUPLICATE_ENUM_BACKING_VALUE};
use fpas_language_service::{LanguageService, WorkspaceContext};
use support::TempDirectory;

#[test]
fn enum_values_overlay_diagnostics_follow_edits_without_changing_disk() {
    let temp = TempDirectory::new("enum-values-overlay");
    let disk = "program T; type Phase = enum First = 1; Second = 2; end enum; begin end.";
    let path = temp.write("enum.fpas", disk);
    let mut service = LanguageService::new(WorkspaceContext::loose(temp.path()));
    for (version, source, expected) in [
        (
            1,
            "program T; type Phase = enum First = 1; Second = 1; end enum; begin end.",
            Some(SEMA_DUPLICATE_ENUM_BACKING_VALUE),
        ),
        (
            2,
            "program T; type Phase = enum First = 1; Second(Value: integer); end enum; begin end.",
            Some(SEMA_BACKING_VALUE_IN_DATA_ENUM),
        ),
        (
            3,
            "program T; type Phase = enum First; Second(Value: integer); end enum; begin end.",
            None,
        ),
    ] {
        if version == 1 {
            service
                .documents_mut()
                .open_document(&path, version, source)
                .expect("open overlay");
        } else {
            service
                .documents_mut()
                .apply_full_text(&path, version, source)
                .expect("edit overlay");
        }
        let analysis = service.analyze_document(&path).expect("overlay analysis");
        match expected {
            Some(code) => {
                assert_eq!(
                    analysis.diagnostics().len(),
                    1,
                    "{:?}",
                    analysis.diagnostics()
                );
                assert_eq!(analysis.diagnostics()[0].code, code);
            }
            None => assert!(
                analysis.diagnostics().is_empty(),
                "{:?}",
                analysis.diagnostics()
            ),
        }
    }
    assert_eq!(std::fs::read_to_string(&path).expect("disk source"), disk);
}
