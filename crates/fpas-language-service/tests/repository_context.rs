#![allow(
    clippy::expect_used,
    clippy::panic,
    reason = "integration fixtures use expect to keep discovery assertions focused"
)]

mod support;

use std::path::Path;

use fpas_language_service::{LanguageService, WorkspaceKind};
use support::TempDirectory;

#[test]
fn repository_root_discovers_the_nested_source_standard_library() {
    let repository_root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let source = repository_root.join("lib/Std/Tui.fpas");
    let mut service = LanguageService::load(&repository_root);

    let analysis = service
        .analyze_document(&source)
        .expect("nested source standard library analysis");

    assert!(
        analysis.diagnostics().is_empty(),
        "{:?}",
        analysis.diagnostics()
    );
    assert_eq!(service.workspace().kind(), WorkspaceKind::Folder);
    assert!(
        service
            .workspace()
            .project_for_source(&source)
            .is_some_and(|project| project.manifest_path().ends_with("stdlib.fpasprj"))
    );
}

#[test]
fn one_repository_session_loads_multiple_nested_projects_lazily() {
    let temp = TempDirectory::new("repository-multiple-projects");
    let first = write_library(
        &temp,
        "repository/first",
        "first",
        "Demo.First",
        "first.fpas",
    );
    let second = write_library(
        &temp,
        "repository/second",
        "second",
        "Demo.Second",
        "second.fpas",
    );
    let mut service = LanguageService::load(&temp.join("repository"));

    let first_analysis = service
        .analyze_document(&first)
        .expect("first nested project analysis");
    let second_analysis = service
        .analyze_document(&second)
        .expect("second nested project analysis");

    assert!(first_analysis.diagnostics().is_empty());
    assert!(second_analysis.diagnostics().is_empty());
    assert_eq!(service.workspace().projects().len(), 2);
}

#[test]
fn direct_owner_replaces_a_previously_loaded_dependency_context() {
    let temp = TempDirectory::new("repository-direct-owner");
    let core_manifest = temp.write(
        "repository/core/core.fpasprj",
        r#"[project]
name = "core"
kind = "library"

[sources]
include = ["src/**/*.fpas"]
"#,
    );
    let core = temp.write(
        "repository/core/src/api.fpas",
        "unit Demo.Api;\n\npublic function Answer(): integer;\nbegin\n  return 42\nend;\n",
    );
    temp.write(
        "repository/app/app.fpasprj",
        r#"[project]
name = "app"
kind = "program"
main = "src/main.fpas"

[dependencies]
projects = ["../core/core.fpasprj"]

[sources]
include = ["src/**/*.fpas"]
"#,
    );
    let main = temp.write(
        "repository/app/src/main.fpas",
        "program App;\n\nuses Demo.Api;\n\nbegin\n  var Value: integer := Answer()\nend.\n",
    );
    let mut service = LanguageService::load(&temp.join("repository"));
    service
        .analyze_document(&main)
        .expect("consumer project analysis");

    service
        .analyze_document(&core)
        .expect("direct library project analysis");

    let selected = service
        .workspace()
        .project_for_source(&core)
        .expect("project owning dependency source");
    assert_eq!(
        std::fs::canonicalize(selected.manifest_path()).expect("selected manifest"),
        std::fs::canonicalize(core_manifest).expect("core manifest")
    );
}

#[test]
fn overlapping_nearest_projects_return_an_actionable_ambiguity() {
    let temp = TempDirectory::new("repository-ambiguous-owner");
    for manifest in ["first.fpasprj", "second.fpasprj"] {
        temp.write(
            format!("repository/shared/{manifest}"),
            &format!(
                r#"[project]
name = "{manifest}"
kind = "library"

[sources]
include = ["shared.fpas"]
"#
            ),
        );
    }
    let source = temp.write("repository/shared/shared.fpas", "unit Demo.Shared;\n");
    let mut service = LanguageService::load(&temp.join("repository"));

    let Err(error) = service.analyze_document(&source) else {
        panic!("overlapping direct project ownership must fail");
    };

    let message = error.to_string();
    assert!(message.contains("multiple FPAS projects"), "{message}");
    assert!(message.contains("first.fpasprj"), "{message}");
    assert!(message.contains("second.fpasprj"), "{message}");
    assert!(message.contains("Adjust `[sources]`"), "{message}");
}

#[test]
fn loose_files_remain_analyzable_after_a_nested_project_is_loaded() {
    let temp = TempDirectory::new("repository-project-and-loose");
    let project_source = write_library(
        &temp,
        "repository/project",
        "project",
        "Demo.Project",
        "project.fpas",
    );
    let loose = temp.write(
        "repository/scratch/loose.fpas",
        "program Loose;\n\nbegin\n  var Value: integer := 1\nend.\n",
    );
    let mut service = LanguageService::load(&temp.join("repository"));
    service
        .analyze_document(&project_source)
        .expect("nested project analysis");

    let analysis = service
        .analyze_document(&loose)
        .expect("loose file analysis");

    assert!(analysis.diagnostics().is_empty());
    assert!(service.workspace().project_for_source(&loose).is_none());
}

fn write_library(
    temp: &TempDirectory,
    root: &str,
    name: &str,
    unit: &str,
    source_name: &str,
) -> std::path::PathBuf {
    temp.write(
        format!("{root}/{name}.fpasprj"),
        &format!(
            r#"[project]
name = "{name}"
kind = "library"

[sources]
include = ["src/**/*.fpas"]
"#
        ),
    );
    temp.write(
        format!("{root}/src/{source_name}"),
        &format!("unit {unit};\n"),
    )
}
