//! Integration tests for project-loading edge cases and diagnostics.

#![allow(
    clippy::expect_used,
    reason = "project loading fixtures use expect for compact setup"
)]

use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

use fpas_project::{load_project, load_workspace};

fn temp_dir(name: &str) -> PathBuf {
    static NEXT: AtomicU64 = AtomicU64::new(1);
    let id = NEXT.fetch_add(1, Ordering::Relaxed);
    std::env::temp_dir().join(format!(
        "fpas-project-loading-{name}-{}-{id}",
        std::process::id()
    ))
}

fn write(path: &Path, text: &str) {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).expect("fixture directory");
    }
    fs::write(path, text).expect("fixture file");
}

#[test]
fn load_project_rejects_cyclic_project_dependency() {
    let dir = temp_dir("project-cycle");
    let lib_a = dir.join("a/a.fpasprj");
    let lib_b = dir.join("b/b.fpasprj");
    write(
        &lib_a,
        "[project]\nname = \"a\"\nkind = \"library\"\n\n[dependencies]\nprojects = [\"../b/b.fpasprj\"]\n\n[sources]\ninclude = [\"src/**/*.fpas\"]\n",
    );
    write(&dir.join("a/src/a.fpas"), "unit Lib.A;\n");
    write(
        &lib_b,
        "[project]\nname = \"b\"\nkind = \"library\"\n\n[dependencies]\nprojects = [\"../a/a.fpasprj\"]\n\n[sources]\ninclude = [\"src/**/*.fpas\"]\n",
    );
    write(&dir.join("b/src/b.fpas"), "unit Lib.B;\n");

    let error = load_project(&lib_a).expect_err("cyclic project deps must fail");
    fs::remove_dir_all(&dir).ok();

    assert!(
        error.contains("Cyclic project dependency detected"),
        "{error}"
    );
}

#[test]
fn load_workspace_rejects_duplicate_member_paths() {
    let dir = temp_dir("workspace-duplicate");
    let workspace = dir.join("suite.fpasworkspace");
    write(
        &dir.join("lib.fpasprj"),
        "[project]\nname = \"lib\"\nkind = \"library\"\n\n[sources]\ninclude = [\"lib.fpas\"]\n",
    );
    write(&dir.join("lib.fpas"), "unit L.Core;\n");
    write(
        &workspace,
        "[workspace]\nname = \"suite\"\nmembers = [\"lib.fpasprj\", \"lib.fpasprj\"]\n",
    );

    let error = load_workspace(&workspace).expect_err("duplicate members must fail");
    fs::remove_dir_all(&dir).ok();

    assert!(error.contains("Duplicate workspace member"), "{error}");
}
