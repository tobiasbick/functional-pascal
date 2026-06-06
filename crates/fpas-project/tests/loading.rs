//! Integration tests for `fpas_project` loading and workspace resolution.
//!
//! Documentation: `docs/pascal/10-projects.md`

#![allow(
    clippy::expect_used,
    clippy::unwrap_used,
    reason = "integration tests use expect/unwrap to keep fixture setup compact"
)]

use fpas_project::{
    LibraryExportPolicy, ProjectKind, SourceOrigin, discover_run_project_in_workspace,
    load_project, load_workspace, resolve_workspace_dependency_paths,
};
use std::fs;
use std::path::Path;
use std::sync::atomic::{AtomicU64, Ordering};

fn temp_dir(name: &str) -> std::path::PathBuf {
    static NEXT: AtomicU64 = AtomicU64::new(1);
    let id = NEXT.fetch_add(1, Ordering::Relaxed);
    std::env::temp_dir().join(format!(
        "fpas-project-tests-{name}-{}-{id}",
        std::process::id()
    ))
}

fn write(path: &Path, text: &str) {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).expect("parent directories must exist");
    }
    fs::write(path, text).expect("test file must be written");
}

#[test]
fn load_project_merges_library_dependency_sources() {
    let dir = temp_dir("dep-merge");
    let lib_project = dir.join("lib/lib.fpasprj");
    let app_project = dir.join("app/app.fpasprj");

    write(
        &lib_project,
        r#"[project]
name = "mylib"
kind = "library"

[sources]
include = ["src/**/*.fpas"]
"#,
    );
    write(
        &lib_project.parent().unwrap().join("src/core.fpas"),
        "unit MyLib.Core;\nconst Value: integer := 7;\n",
    );

    write(
        &app_project,
        r#"[project]
name = "app"
kind = "program"
main = "src/main.fpas"

[dependencies]
projects = ["../lib/lib.fpasprj"]

[sources]
include = ["src/**/*.fpas"]
"#,
    );
    write(
        &app_project.parent().unwrap().join("src/main.fpas"),
        "program App;\nbegin\nend.\n",
    );

    let loaded = load_project(&app_project).expect("project should load");
    fs::remove_dir_all(&dir).ok();

    assert_eq!(loaded.kind, ProjectKind::Program);
    assert_eq!(loaded.source_files.len(), 1);
}

#[test]
fn load_project_applies_sources_exclude() {
    let dir = temp_dir("exclude");
    let project = dir.join("app.fpasprj");

    write(
        &project,
        r#"[project]
name = "app"
kind = "program"
main = "src/main.fpas"

[sources]
include = ["src/**/*.fpas"]
exclude = ["src/generated/**/*.fpas"]
"#,
    );
    write(&dir.join("src/main.fpas"), "program App;\nbegin\nend.\n");
    write(&dir.join("src/generated/stub.fpas"), "unit App.Gen;\n");
    write(&dir.join("src/live.fpas"), "unit App.Live;\n");

    let loaded = load_project(&project).expect("project should load");
    fs::remove_dir_all(&dir).ok();

    assert_eq!(loaded.source_files.len(), 1);
    assert!(
        loaded
            .source_files
            .iter()
            .any(|path| path.file_name().is_some_and(|name| name == "live.fpas"))
    );
}

#[test]
fn resolve_workspace_dependency_paths_finds_member_by_name() {
    let dir = temp_dir("workspace-dep");
    let workspace = dir.join("suite.fpasworkspace");
    let lib = dir.join("libs/greet/greet.fpasprj");
    let app = dir.join("apps/hello/hello.fpasprj");

    write(
        &workspace,
        r#"[workspace]
name = "suite"
members = ["libs/greet/greet.fpasprj", "apps/hello/hello.fpasprj"]
"#,
    );
    write(
        &lib,
        r#"[project]
name = "greet"
kind = "library"

[sources]
include = ["src/**/*.fpas"]
"#,
    );
    write(
        &lib.parent().unwrap().join("src/greet.fpas"),
        "unit Demo.Greet;\n",
    );
    write(
        &app,
        r#"[project]
name = "hello"
kind = "program"
main = "src/main.fpas"

[sources]
include = ["src/**/*.fpas"]
"#,
    );
    write(
        &app.parent().unwrap().join("src/main.fpas"),
        "program Hello;\nbegin\nend.\n",
    );

    let paths =
        resolve_workspace_dependency_paths(&app, &["greet".to_string()]).expect("resolve dep");
    fs::remove_dir_all(&dir).ok();

    assert_eq!(paths.len(), 1);
    assert!(paths[0].ends_with("greet.fpasprj"));
}

#[test]
fn discover_run_project_in_workspace_returns_single_program() {
    let dir = temp_dir("discover-run");
    let workspace = dir.join("suite.fpasworkspace");

    write(
        &workspace,
        r#"[workspace]
name = "suite"
members = ["lib.fpasprj", "app.fpasprj"]
"#,
    );
    write(
        &dir.join("lib.fpasprj"),
        r#"[project]
name = "lib"
kind = "library"

[sources]
include = ["lib.fpas"]
"#,
    );
    write(&dir.join("lib.fpas"), "unit L.Core;\n");
    write(
        &dir.join("app.fpasprj"),
        r#"[project]
name = "app"
kind = "program"
main = "main.fpas"

[sources]
include = ["main.fpas"]
"#,
    );
    write(&dir.join("main.fpas"), "program App;\nbegin\nend.\n");

    let program = discover_run_project_in_workspace(&workspace).expect("discover program");
    let members = load_workspace(&workspace)
        .expect("load workspace")
        .member_projects;
    fs::remove_dir_all(&dir).ok();

    assert_eq!(members.len(), 2);
    assert!(program.ends_with("app.fpasprj"));
}

#[test]
fn discover_run_project_in_workspace_errors_when_no_program() {
    let dir = temp_dir("discover-no-program");
    let workspace = dir.join("suite.fpasworkspace");

    write(
        &workspace,
        r#"[workspace]
name = "suite"
members = ["lib.fpasprj"]
"#,
    );
    write(
        &dir.join("lib.fpasprj"),
        r#"[project]
name = "lib"
kind = "library"

[sources]
include = ["lib.fpas"]
"#,
    );
    write(&dir.join("lib.fpas"), "unit L.Core;\n");

    let error = discover_run_project_in_workspace(&workspace).expect_err("must fail");
    fs::remove_dir_all(&dir).ok();

    assert!(error.contains("No `program` projects found"));
}

#[test]
fn load_project_rejects_unknown_export_unit_name() {
    let dir = temp_dir("exports-unknown-unit");
    let project = dir.join("lib.fpasprj");

    write(
        &project,
        r#"[project]
name = "lib"
kind = "library"

[exports]
units = ["Missing.Unit"]

[sources]
include = ["src/**/*.fpas"]
"#,
    );
    write(&dir.join("src/core.fpas"), "unit Lib.Core;\n");

    let error = load_project(&project).expect_err("unknown export unit must fail");
    fs::remove_dir_all(&dir).ok();

    assert!(
        error.contains("exports.units") && error.contains("unknown unit"),
        "got: {error}"
    );
}

#[test]
fn load_project_preserves_transitive_library_export_policies() {
    let dir = temp_dir("transitive-export-meta");
    let base_dir = dir.join("libs/base");
    let util_dir = dir.join("libs/util");
    let app_dir = dir.join("apps/demo");
    let base_project = base_dir.join("base.fpasprj");
    let util_project = util_dir.join("util.fpasprj");
    let app_project = app_dir.join("demo.fpasprj");

    write(
        &base_project,
        r#"[project]
name = "base"
kind = "library"

[exports]
units = ["Lib.Base"]

[sources]
include = ["src/**/*.fpas"]
"#,
    );
    write(&base_dir.join("src/base.fpas"), "unit Lib.Base;\n");
    write(
        &base_dir.join("src/internal.fpas"),
        "unit Lib.Base.Internal;\n",
    );

    write(
        &util_project,
        r#"[project]
name = "util"
kind = "library"

[dependencies]
projects = ["../base/base.fpasprj"]

[sources]
include = ["src/**/*.fpas"]
"#,
    );
    write(
        &util_dir.join("src/util.fpas"),
        "unit Lib.Util;\nuses Lib.Base;\n",
    );

    write(
        &app_project,
        r#"[project]
name = "demo"
kind = "program"
main = "src/main.fpas"

[dependencies]
projects = ["../../libs/util/util.fpasprj"]

[sources]
include = ["src/**/*.fpas"]
"#,
    );
    write(
        &app_dir.join("src/main.fpas"),
        "program Demo;\nuses Lib.Util;\nbegin\nend.\n",
    );

    let loaded = load_project(&app_project).expect("project should load");
    let internal_key = loaded
        .source_files
        .iter()
        .find(|path| path.file_name().is_some_and(|name| name == "internal.fpas"))
        .expect("internal source file must be merged");

    assert!(loaded.link_meta.enforces_export_rules());
    assert!(matches!(
        loaded.link_meta.origin_for_source(internal_key),
        SourceOrigin::Library(path) if path.ends_with("base.fpasprj")
    ));
    assert!(
        loaded
            .link_meta
            .library_export_policies
            .values()
            .any(|policy| {
                matches!(
                    policy,
                    LibraryExportPolicy::ListedUnits(units) if units.contains("lib.base")
                )
            }),
        "expected base export policy, got: {:?}",
        loaded.link_meta.library_export_policies
    );

    fs::remove_dir_all(&dir).ok();
}
