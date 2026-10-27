//! Snapshot ownership and rejection rules for the public unit view.

#![allow(clippy::expect_used, reason = "fixtures use direct assertions")]
mod support;

use fpas_language_service::{LanguageService, SourceVersion, WorkspaceContext};
use support::TempDirectory;

#[test]
fn public_view_uses_unsaved_source_without_writing_the_unit_or_sidecars() {
    let temp = TempDirectory::new("public-view");
    let original = "unit Demo;\npublic const Original: integer := 1;\nend unit;\n";
    let path = temp.write("unit.fpas", original);
    let mut service = LanguageService::new(WorkspaceContext::loose(temp.path()));
    let edited = original.replace("Original", "Unsaved");
    service
        .documents_mut()
        .open_document(&path, 7, edited.clone())
        .expect("overlay");
    let result = service.public_signatures(&path).expect("public view");
    assert_eq!(result.snapshot.version(), SourceVersion::Editor(7));
    assert_eq!(result.snapshot.source(), edited);
    assert!(result.value.contains("public const Unsaved: integer;"));
    assert_eq!(
        std::fs::read_to_string(&path).expect("disk source"),
        original
    );
    assert!(!path.with_extension("fpascu").exists());
}

#[test]
fn public_view_rejects_programs_and_recovered_invalid_units() {
    let temp = TempDirectory::new("invalid-public-view");
    let mut service = LanguageService::new(WorkspaceContext::loose(temp.path()));
    for (name, source, expected) in [
        ("main", "program Main; begin end.", "unit sources"),
        (
            "broken",
            "unit Broken; public function Missing(",
            "syntax errors",
        ),
    ] {
        let path = temp.write(format!("{name}.fpas"), source);
        let error = service.public_signatures(&path).expect_err("must reject");
        assert!(error.to_string().contains(expected), "{error}");
    }
}

#[test]
fn equality_constraints_work_in_unsaved_analysis_and_public_signatures() {
    let temp = TempDirectory::new("equatable-public-view");
    let path = temp.write("equal.fpas", "unit Demo; end unit;");
    let mut service = LanguageService::new(WorkspaceContext::loose(temp.path()));
    let source = "unit Demo; public function Same<T: Equatable>(A: T; B: T): boolean; begin return A = B; end function; end unit;";
    service
        .documents_mut()
        .open_document(&path, 1, source)
        .expect("overlay");
    assert!(
        service
            .analyze_document(&path)
            .expect("analysis")
            .diagnostics()
            .is_empty()
    );
    let view = service.public_signatures(&path).expect("public signature");
    assert!(view.value.contains("Same<T: Equatable>"), "{}", view.value);
    service
        .documents_mut()
        .open_document(&path, 2, source.replace("A = B", "A < B"))
        .expect("changed overlay");
    assert!(
        !service
            .analyze_document(&path)
            .expect("invalid ordering")
            .diagnostics()
            .is_empty()
    );
}
