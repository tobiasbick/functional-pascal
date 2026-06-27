//! Integration tests for project linking (`build_program`, library check).
//!
//! Documentation: `docs/pascal/program-structure/projects.md`

#![allow(
    clippy::expect_used,
    clippy::unwrap_used,
    reason = "integration tests use expect/unwrap to keep fixture setup compact"
)]

use fpas_parser::Decl;
use fpas_project::{
    build_library_check_with_source_map, build_program, load_project, load_workspace,
};
use std::fs;
use std::path::Path;
use std::sync::atomic::{AtomicU64, Ordering};

fn temp_dir(name: &str) -> std::path::PathBuf {
    static NEXT: AtomicU64 = AtomicU64::new(1);
    let id = NEXT.fetch_add(1, Ordering::Relaxed);
    std::env::temp_dir().join(format!(
        "fpas-project-link-tests-{name}-{}-{id}",
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
fn build_program_rejects_non_exported_cross_library_import() {
    let dir = temp_dir("export-block");
    let lib_dir = dir.join("lib");
    let app_dir = dir.join("app");
    let lib_project = lib_dir.join("lib.fpasprj");
    let app_project = app_dir.join("app.fpasprj");

    write(
        &lib_project,
        r#"[project]
name = "lib"
kind = "library"

[exports]
units = ["MyLib.Core"]

[sources]
include = ["src/**/*.fpas"]
"#,
    );
    write(
        &lib_dir.join("src/core.fpas"),
        "unit MyLib.Core;\nuses MyLib.Internal;\n",
    );
    write(&lib_dir.join("src/internal.fpas"), "unit MyLib.Internal;\n");

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
        &app_dir.join("src/main.fpas"),
        "program App;\nuses MyLib.Internal;\nbegin\nend.\n",
    );

    let loaded = load_project(&app_project).expect("project should load");
    let error = build_program(
        loaded.main.as_deref().expect("main"),
        &loaded.source_files,
        &loaded.link_meta,
    )
    .expect_err("non-exported unit must fail at link");
    fs::remove_dir_all(&dir).ok();

    assert!(
        error.contains("not exported"),
        "expected export error, got: {error}"
    );
}

#[test]
fn build_program_reports_cyclic_unit_dependency() {
    let dir = temp_dir("unit-cycle");
    let project = dir.join("app.fpasprj");

    write(
        &project,
        r#"[project]
name = "app"
kind = "program"
main = "src/main.fpas"

[sources]
include = ["src/**/*.fpas"]
"#,
    );
    write(&dir.join("src/a.fpas"), "unit App.A;\nuses App.B;\n");
    write(&dir.join("src/b.fpas"), "unit App.B;\nuses App.A;\n");
    write(
        &dir.join("src/main.fpas"),
        "program App;\nuses App.A;\nbegin\nend.\n",
    );

    let loaded = load_project(&project).expect("project should load");
    let error = build_program(
        loaded.main.as_deref().expect("main"),
        &loaded.source_files,
        &loaded.link_meta,
    )
    .expect_err("cyclic units must fail at link");
    fs::remove_dir_all(&dir).ok();

    assert!(
        error.contains("Cyclic unit dependency detected"),
        "got: {error}"
    );
}

#[test]
fn load_project_rejects_cyclic_project_dependency() {
    let dir = temp_dir("project-cycle");
    let lib_a = dir.join("a/a.fpasprj");
    let lib_b = dir.join("b/b.fpasprj");

    write(
        &lib_a,
        r#"[project]
name = "a"
kind = "library"

[dependencies]
projects = ["../b/b.fpasprj"]

[sources]
include = ["src/**/*.fpas"]
"#,
    );
    write(&dir.join("a/src/a.fpas"), "unit Lib.A;\n");
    write(
        &lib_b,
        r#"[project]
name = "b"
kind = "library"

[dependencies]
projects = ["../a/a.fpasprj"]

[sources]
include = ["src/**/*.fpas"]
"#,
    );
    write(&dir.join("b/src/b.fpas"), "unit Lib.B;\n");

    let error = load_project(&lib_a).expect_err("cyclic project deps must fail");
    fs::remove_dir_all(&dir).ok();

    assert!(
        error.contains("Cyclic project dependency detected"),
        "got: {error}"
    );
}

#[test]
fn build_library_check_reserves_source_id_zero_for_stub() {
    let dir = temp_dir("lib-check-ids");
    let project = dir.join("lib.fpasprj");

    write(
        &project,
        r#"[project]
name = "lib"
kind = "library"

[sources]
include = ["src/**/*.fpas"]
"#,
    );
    write(
        &dir.join("src/one.fpas"),
        "unit Lib.One;\nconst A: integer := 1;\n",
    );
    write(
        &dir.join("src/two.fpas"),
        "unit Lib.Two;\nconst B: integer := 2;\n",
    );

    let loaded = load_project(&project).expect("library should load");
    let linked = build_library_check_with_source_map(&loaded.source_files, &loaded.link_meta)
        .expect("library check should link");
    fs::remove_dir_all(&dir).ok();

    assert_eq!(linked.source_paths.len(), 3);
    assert_eq!(
        linked.source_paths[0].to_string_lossy(),
        "<fpas-library-check>"
    );
    assert_eq!(linked.program.name_span.source_id, 0);

    let mut unit_source_ids = Vec::new();
    for decl in &linked.program.declarations {
        if let Decl::Const(c) = decl {
            unit_source_ids.push(c.span.source_id);
        }
    }
    unit_source_ids.sort_unstable();
    assert_eq!(unit_source_ids, vec![1, 2]);
}

#[test]
fn load_workspace_rejects_duplicate_member_paths() {
    let dir = temp_dir("workspace-dup");
    let workspace = dir.join("suite.fpasworkspace");
    let project = dir.join("lib.fpasprj");

    write(
        &project,
        r#"[project]
name = "lib"
kind = "library"

[sources]
include = ["lib.fpas"]
"#,
    );
    write(&dir.join("lib.fpas"), "unit L.Core;\n");
    write(
        &workspace,
        r#"[workspace]
name = "suite"
members = ["lib.fpasprj", "lib.fpasprj"]
"#,
    );

    let error = load_workspace(&workspace).expect_err("duplicate members must fail");
    fs::remove_dir_all(&dir).ok();

    assert!(error.contains("Duplicate workspace member"), "got: {error}");
}
