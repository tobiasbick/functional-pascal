//! Integration tests for workspace manifests and member context.

#![allow(
    clippy::expect_used,
    reason = "integration fixtures use expect to keep manifest assertions focused"
)]

mod support;

use fpas_language_service::{WorkspaceContext, WorkspaceKind};
use support::{TempDirectory, write_program_project};

#[test]
fn explicit_project_and_source_discovery_load_the_same_project() {
    let temp = TempDirectory::new("workspace-project");
    let (manifest, main, unit) = write_program_project(&temp);

    let explicit = WorkspaceContext::load(&manifest);
    assert_eq!(explicit.kind(), WorkspaceKind::Project);
    assert!(explicit.issues().is_empty());
    assert_eq!(explicit.projects().len(), 1);
    assert!(explicit.project_for_source(&main).is_some());
    assert!(explicit.project_for_source(&unit).is_some());

    let discovered = WorkspaceContext::load(&main);
    assert_eq!(discovered.kind(), WorkspaceKind::Project);
    assert_eq!(discovered.manifest_path(), explicit.manifest_path());
}

#[test]
fn workspace_loads_usable_members_and_records_invalid_member() {
    let temp = TempDirectory::new("workspace-members");
    let valid_manifest = temp.write(
        "valid/valid.fpasprj",
        r#"[project]
name = "valid"
kind = "library"

[sources]
include = ["src/**/*.fpas"]
"#,
    );
    temp.write("valid/src/valid.fpas", "unit Demo.Valid;\n");
    let invalid_manifest = temp.write("invalid/invalid.fpasprj", "not toml");
    let workspace = temp.write(
        "suite.fpasworkspace",
        r#"[workspace]
name = "suite"
members = ["valid/valid.fpasprj", "invalid/invalid.fpasprj"]
"#,
    );

    let context = WorkspaceContext::load(&workspace);
    assert_eq!(context.kind(), WorkspaceKind::Workspace);
    assert_eq!(context.projects().len(), 1);
    assert_eq!(
        std::fs::canonicalize(context.projects()[0].manifest_path())
            .expect("canonical loaded manifest"),
        std::fs::canonicalize(valid_manifest).expect("canonical valid manifest"),
    );
    assert_eq!(context.issues().len(), 1);
    assert_eq!(
        std::fs::canonicalize(&context.issues()[0].path)
            .expect("canonical loaded invalid manifest"),
        std::fs::canonicalize(invalid_manifest).expect("canonical invalid manifest"),
    );
}

#[test]
fn invalid_manifest_and_missing_dependency_are_recoverable_states() {
    let temp = TempDirectory::new("workspace-errors");
    let invalid = temp.write("invalid.fpasprj", "[project\n");
    let invalid_context = WorkspaceContext::load(&invalid);
    assert_eq!(invalid_context.kind(), WorkspaceKind::Unavailable);
    assert_eq!(invalid_context.issues().len(), 1);
    assert!(invalid_context.projects().is_empty());

    let missing_dependency = temp.write(
        "missing.fpasprj",
        r#"[project]
name = "missing"
kind = "program"
main = "main.fpas"

[dependencies]
projects = ["does-not-exist.fpasprj"]

[sources]
include = ["*.fpas"]
"#,
    );
    temp.write("main.fpas", "program Missing;\nbegin\nend.\n");
    let missing_context = WorkspaceContext::load(&missing_dependency);
    assert_eq!(missing_context.kind(), WorkspaceKind::Unavailable);
    assert!(
        missing_context.issues()[0]
            .message
            .contains("does-not-exist.fpasprj")
    );
}

#[test]
fn source_without_metadata_uses_loose_context() {
    let temp = TempDirectory::new("workspace-loose");
    let source = temp.write("loose.fpas", "program Loose;\nbegin\nend.\n");

    let context = WorkspaceContext::load(&source);

    assert_eq!(context.kind(), WorkspaceKind::Loose);
    assert!(context.projects().is_empty());
    assert!(context.issues().is_empty());
}

#[test]
fn source_discovery_preserves_invalid_ancestor_manifest_errors() {
    let temp = TempDirectory::new("ancestor-invalid");
    let source = temp.write("src/main.fpas", "program App; begin end.");
    for (extension, text) in [("fpasprj", "[project\n"), ("fpasworkspace", "[workspace\n")] {
        let manifest = temp.write(format!("broken.{extension}"), text);
        let context = WorkspaceContext::load(&source);
        assert_eq!(context.kind(), WorkspaceKind::Unavailable);
        assert_eq!(context.issues().len(), 1);
        assert_eq!(context.issues()[0].path, manifest);
        assert_diagnostic_failure(context, &source);
        std::fs::remove_file(manifest).expect("remove invalid manifest");
    }
}

#[test]
fn source_discovery_preserves_missing_ancestor_dependency_errors() {
    let temp = TempDirectory::new("ancestor-dependency");
    let source = temp.write("src/main.fpas", "program App; begin end.");
    temp.write("app.fpasprj", "[project]\nname = \"app\"\nkind = \"program\"\nmain = \"src/main.fpas\"\n[dependencies]\nprojects = [\"missing.fpasprj\"]\n[sources]\ninclude = [\"src/*.fpas\"]\n");
    let context = WorkspaceContext::load(&source);
    assert_eq!(context.kind(), WorkspaceKind::Unavailable);
    assert!(
        context.issues()[0].message.contains("missing.fpasprj"),
        "{:?}",
        context.issues()
    );
    assert_diagnostic_failure(context, &source);
}

#[test]
fn source_discovery_preserves_ambiguous_ancestor_ownership() {
    let temp = TempDirectory::new("ancestor-owners");
    let source = temp.write("src/main.fpas", "program App; begin end.");
    for name in ["one", "two"] {
        temp.write(
            format!("{name}.fpasprj"),
            &format!(
                "[project]\nname = \"{name}\"\nkind = \"program\"\nmain = \"src/main.fpas\"\n[sources]\ninclude = [\"src/*.fpas\"]\n"
            ),
        );
    }
    let context = WorkspaceContext::load(&source);
    assert_eq!(context.kind(), WorkspaceKind::Unavailable);
    assert!(
        context.issues()[0]
            .message
            .contains("multiple FPAS projects"),
        "{:?}",
        context.issues()
    );
    assert_diagnostic_failure(context, &source);
}

fn assert_diagnostic_failure(context: WorkspaceContext, source: &std::path::Path) {
    let mut service = fpas_language_service::LanguageService::new(context);
    for _ in 0..2 {
        let analysis = service
            .analyze_document_diagnostics(source)
            .expect("recoverable diagnostics");
        assert!(
            analysis.failure().is_some(),
            "a later query must preserve the project error"
        );
    }
}
