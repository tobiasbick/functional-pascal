//! Integration tests for indexing project symbols and dependencies.

#![allow(
    clippy::expect_used,
    reason = "integration fixtures use expect to keep index assertions focused"
)]

mod support;

use fpas_language_service::{CancellationToken, LanguageService, LanguageServiceError};
use support::TempDirectory;

#[test]
fn folder_catalog_refreshes_dependencies_and_is_open_order_independent() {
    let temp = TempDirectory::new("project-index-dependency");
    let declaration = write_core(&temp);
    let manifest = temp.write("app/app.fpasprj", app_manifest(false));
    let consumer = temp.write(
        "app/src/main.fpas",
        "program App;\n\nuses Demo.Core;\n\nbegin\n  var Value: integer := Answer()\nend.\n",
    );
    let unrelated = temp.write(
        "unrelated/src/main.fpas",
        "program Unrelated;\n\nfunction Answer(): integer;\nbegin\n  return 7\nend;\n\nbegin\n  var Value: integer := Answer()\nend.\n",
    );
    temp.write(
        "unrelated/unrelated.fpasprj",
        r#"[project]
name = "unrelated"
kind = "program"
main = "src/main.fpas"

[sources]
include = ["src/**/*.fpas"]
"#,
    );

    let offset = std::fs::read_to_string(&declaration)
        .expect("core source")
        .find("Answer()")
        .expect("Answer declaration");
    let mut service = LanguageService::load(temp.path());
    assert_eq!(service.workspace().projects().len(), 3);
    assert!(
        service
            .references(&declaration, offset, false)
            .expect("references before dependency")
            .value
            .is_empty()
    );

    std::fs::write(&manifest, app_manifest(true)).expect("dependency manifest update");
    service
        .refresh_paths(std::slice::from_ref(&manifest), &CancellationToken::new())
        .expect("project index refresh");
    let refreshed = service
        .references(&declaration, offset, false)
        .expect("references after dependency")
        .value;
    assert_eq!(refreshed.len(), 1, "{refreshed:?}");
    assert_eq!(refreshed[0].path, consumer);
    assert!(
        refreshed
            .iter()
            .all(|reference| reference.path != unrelated)
    );

    let mut fresh = LanguageService::load(temp.path());
    let fresh_paths = fresh
        .references(&declaration, offset, false)
        .expect("fresh references")
        .value;
    assert_eq!(fresh_paths, refreshed);
}

#[test]
fn source_creation_and_deletion_update_glob_membership() {
    let temp = TempDirectory::new("project-index-sources");
    write_core(&temp);
    let created = temp.write("core/src/extra.fpas", "unit Demo.Extra;\n");
    let mut service = LanguageService::load(temp.path());
    assert!(service.workspace().project_for_source(&created).is_some());

    std::fs::remove_file(&created).expect("delete indexed source");
    service
        .refresh_paths(std::slice::from_ref(&created), &CancellationToken::new())
        .expect("refresh deleted source");
    assert!(service.workspace().project_for_source(&created).is_none());

    std::fs::write(&created, "unit Demo.Extra;\n").expect("recreate indexed source");
    service
        .refresh_paths(std::slice::from_ref(&created), &CancellationToken::new())
        .expect("refresh created source");
    assert!(service.workspace().project_for_source(&created).is_some());
}

#[test]
fn cancelled_refresh_keeps_the_previous_complete_catalog() {
    let temp = TempDirectory::new("project-index-cancel");
    let declaration = write_core(&temp);
    let mut service = LanguageService::load(temp.path());
    let previous = service.workspace().projects().len();
    let cancellation = CancellationToken::new();
    cancellation.cancel();

    let error = service
        .refresh_paths(&[declaration], &cancellation)
        .expect_err("cancelled refresh");

    assert_eq!(error, LanguageServiceError::Cancelled);
    assert_eq!(service.workspace().projects().len(), previous);
}

#[test]
fn refresh_preserves_authoritative_open_snapshot() {
    let temp = TempDirectory::new("project-index-open-snapshot");
    let source = write_core(&temp);
    let mut service = LanguageService::load(temp.path());
    let unsaved = "unit Demo.Core;\n\n// unsaved editor text\n";
    service
        .documents_mut()
        .open_document(&source, 7, unsaved)
        .expect("open editor snapshot");
    std::fs::write(&source, "unit Demo.Core;\n\n// external disk text\n")
        .expect("external source update");

    service
        .refresh_paths(std::slice::from_ref(&source), &CancellationToken::new())
        .expect("refresh external source update");

    assert_eq!(
        service
            .snapshot(&source)
            .expect("current snapshot")
            .source(),
        unsaved
    );
}

#[test]
fn source_create_and_delete_refresh_project_analysis() {
    let temp = TempDirectory::new("project-index-diagnostics");
    temp.write(
        "core/core.fpasprj",
        "[project]\nname = \"core\"\nkind = \"library\"\n\n[sources]\ninclude = [\"src/**/*.fpas\"]\n",
    );
    temp.write("core/src/other.fpas", "unit Demo.Other;\n");
    temp.write("app/app.fpasprj", app_manifest(true));
    let main = temp.write(
        "app/src/main.fpas",
        "program App;\n\nuses Demo.Core;\n\nbegin var Value: integer := Answer() end.\n",
    );
    let mut service = LanguageService::load(temp.path());
    assert!(service.analyze_document(&main).is_err());

    let core = temp.write(
        "core/src/core.fpas",
        "unit Demo.Core;\n\npublic function Answer(): integer;\nbegin return 42 end;\n",
    );
    service
        .refresh_paths(std::slice::from_ref(&core), &CancellationToken::new())
        .expect("refresh created dependency source");
    assert!(
        service
            .analyze_document(&main)
            .expect("analysis after source creation")
            .diagnostics()
            .is_empty()
    );

    std::fs::remove_file(&core).expect("delete dependency source");
    service
        .refresh_paths(&[core], &CancellationToken::new())
        .expect("refresh deleted dependency source");
    assert!(service.analyze_document(&main).is_err());
}

#[test]
fn export_changes_refresh_consumer_navigation() {
    let temp = TempDirectory::new("project-index-exports");
    let core_manifest_path = temp.write("core/core.fpasprj", &core_manifest("Demo.Core"));
    let declaration = temp.write(
        "core/src/core.fpas",
        "unit Demo.Core;\n\npublic function Answer(): integer;\nbegin return 42 end;\n",
    );
    temp.write("core/src/other.fpas", "unit Demo.Other;\n");
    temp.write("app/app.fpasprj", app_manifest(true));
    temp.write(
        "app/src/main.fpas",
        "program App;\n\nuses Demo.Core;\n\nbegin var Value: integer := Answer() end.\n",
    );
    let offset = std::fs::read_to_string(&declaration)
        .expect("core source")
        .find("Answer")
        .expect("Answer declaration");
    let mut service = LanguageService::load(temp.path());
    assert_eq!(
        service
            .references(&declaration, offset, false)
            .expect("exported references")
            .value
            .len(),
        1
    );

    std::fs::write(&core_manifest_path, core_manifest("Demo.Other")).expect("remove core export");
    service
        .refresh_paths(
            std::slice::from_ref(&core_manifest_path),
            &CancellationToken::new(),
        )
        .expect("refresh removed export");
    assert!(
        service
            .references(&declaration, offset, false)
            .expect("unexported references")
            .value
            .is_empty()
    );

    std::fs::write(&core_manifest_path, core_manifest("Demo.Core")).expect("restore core export");
    service
        .refresh_paths(&[core_manifest_path], &CancellationToken::new())
        .expect("refresh restored export");
    assert_eq!(
        service
            .references(&declaration, offset, false)
            .expect("restored references")
            .value
            .len(),
        1
    );
}

fn write_core(temp: &TempDirectory) -> std::path::PathBuf {
    temp.write(
        "core/core.fpasprj",
        r#"[project]
name = "core"
kind = "library"

[sources]
include = ["src/**/*.fpas"]
"#,
    );
    temp.write(
        "core/src/core.fpas",
        "unit Demo.Core;\n\npublic function Answer(): integer;\nbegin\n  return 42\nend;\n",
    )
}

fn app_manifest(with_dependency: bool) -> &'static str {
    if with_dependency {
        r#"[project]
name = "app"
kind = "program"
main = "src/main.fpas"

[dependencies]
projects = ["../core/core.fpasprj"]

[sources]
include = ["src/**/*.fpas"]
"#
    } else {
        r#"[project]
name = "app"
kind = "program"
main = "src/main.fpas"

[sources]
include = ["src/**/*.fpas"]
"#
    }
}

fn core_manifest(export: &str) -> String {
    format!(
        "[project]\nname = \"core\"\nkind = \"library\"\n\n[exports]\nunits = [\"{export}\"]\n\n[sources]\ninclude = [\"src/**/*.fpas\"]\n"
    )
}
