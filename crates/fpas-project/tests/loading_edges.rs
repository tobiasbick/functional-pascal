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
fn load_project_rejects_source_owned_by_consumer_and_dependency() {
    let dir = temp_dir("consumer-dependency-source-overlap");
    let library = dir.join("lib/lib.fpasprj");
    let consumer = dir.join("app/app.fpasprj");
    write(
        &library,
        "[project]\nname = \"lib\"\nkind = \"library\"\n\n[exports]\nunits = [\"Lib.Api\"]\n\n[sources]\ninclude = [\"src/*.fpas\"]\n",
    );
    write(&dir.join("lib/src/api.fpas"), "unit Lib.Api;\n");
    write(&dir.join("lib/src/internal.fpas"), "unit Lib.Internal;\n");
    write(
        &consumer,
        "[project]\nname = \"app\"\nkind = \"program\"\nmain = \"main.fpas\"\n\n[dependencies]\nprojects = [\"../lib/lib.fpasprj\"]\n\n[sources]\ninclude = [\"main.fpas\", \"../lib/src/internal.fpas\"]\n",
    );
    write(&dir.join("app/main.fpas"), "program App; begin end.\n");

    let error = load_project(&consumer).expect_err("overlapping source ownership must fail");
    fs::remove_dir_all(&dir).ok();

    assert!(
        error.contains("owned by more than one project")
            && error.contains("internal.fpas")
            && error.contains("lib.fpasprj")
            && error.contains("app.fpasprj"),
        "{error}"
    );
}

#[test]
fn load_project_rejects_lexical_alias_of_dependency_source() {
    let dir = temp_dir("dependency-source-lexical-alias");
    let library = dir.join("lib/lib.fpasprj");
    let consumer = dir.join("app/app.fpasprj");
    write(
        &library,
        "[project]\nname = \"lib\"\nkind = \"library\"\n\n[sources]\ninclude = [\"src/shared.fpas\"]\n",
    );
    write(&dir.join("lib/src/shared.fpas"), "unit Lib.Shared;\n");
    write(
        &consumer,
        "[project]\nname = \"app\"\nkind = \"program\"\nmain = \"main.fpas\"\n\n[dependencies]\nprojects = [\"../lib/lib.fpasprj\"]\n\n[sources]\ninclude = [\"main.fpas\", \"../lib/src/../src/shared.fpas\"]\n",
    );
    write(&dir.join("app/main.fpas"), "program App; begin end.\n");

    let error = load_project(&consumer).expect_err("source alias ownership must fail");
    fs::remove_dir_all(&dir).ok();

    assert!(error.contains("owned by more than one project"), "{error}");
}

#[test]
fn load_project_rejects_symlink_alias_of_dependency_source() {
    let dir = temp_dir("dependency-source-symlink-alias");
    let library = dir.join("lib/lib.fpasprj");
    let consumer = dir.join("app/app.fpasprj");
    let source = dir.join("lib/shared.fpas");
    let alias = dir.join("app/shared_alias.fpas");
    write(
        &library,
        "[project]\nname = \"lib\"\nkind = \"library\"\n\n[sources]\ninclude = [\"shared.fpas\"]\n",
    );
    write(&source, "unit Lib.Shared;\n");
    fs::create_dir_all(alias.parent().expect("alias parent")).expect("alias directory");
    if create_file_symlink(&source, &alias).is_err() {
        fs::remove_dir_all(dir).ok();
        return;
    }
    write(
        &consumer,
        "[project]\nname = \"app\"\nkind = \"program\"\nmain = \"main.fpas\"\n\n[dependencies]\nprojects = [\"../lib/lib.fpasprj\"]\n\n[sources]\ninclude = [\"main.fpas\", \"shared_alias.fpas\"]\n",
    );
    write(&dir.join("app/main.fpas"), "program App; begin end.\n");

    let error = load_project(&consumer).expect_err("source symlink ownership must fail");
    fs::remove_dir_all(&dir).ok();

    assert!(error.contains("owned by more than one project"), "{error}");
}

#[test]
fn load_project_rejects_source_owned_by_two_libraries() {
    let dir = temp_dir("two-library-source-overlap");
    let library_a = dir.join("a/a.fpasprj");
    let library_b = dir.join("b/b.fpasprj");
    let consumer = dir.join("app/app.fpasprj");
    write(&dir.join("shared.fpas"), "unit Lib.Shared;\n");
    write(
        &library_a,
        "[project]\nname = \"a\"\nkind = \"library\"\n\n[sources]\ninclude = [\"../shared.fpas\"]\n",
    );
    write(
        &library_b,
        "[project]\nname = \"b\"\nkind = \"library\"\n\n[sources]\ninclude = [\"../shared.fpas\"]\n",
    );
    write(
        &consumer,
        "[project]\nname = \"app\"\nkind = \"program\"\nmain = \"main.fpas\"\n\n[dependencies]\nprojects = [\"../a/a.fpasprj\", \"../b/b.fpasprj\"]\n\n[sources]\ninclude = [\"main.fpas\"]\n",
    );
    write(&dir.join("app/main.fpas"), "program App; begin end.\n");

    let error = load_project(&consumer).expect_err("two library owners must fail");
    fs::remove_dir_all(&dir).ok();

    assert!(
        error.contains("owned by more than one project")
            && error.contains("a.fpasprj")
            && error.contains("b.fpasprj"),
        "{error}"
    );
}

#[test]
fn load_project_allows_same_library_reached_through_multiple_dependencies() {
    let dir = temp_dir("diamond-library-dependency");
    let base = dir.join("base/base.fpasprj");
    let wrapper = dir.join("wrapper/wrapper.fpasprj");
    let consumer = dir.join("app/app.fpasprj");
    write(
        &base,
        "[project]\nname = \"base\"\nkind = \"library\"\n\n[sources]\ninclude = [\"base.fpas\"]\n",
    );
    write(&dir.join("base/base.fpas"), "unit Lib.Base;\n");
    write(
        &wrapper,
        "[project]\nname = \"wrapper\"\nkind = \"library\"\n\n[dependencies]\nprojects = [\"../base/base.fpasprj\"]\n\n[sources]\ninclude = [\"wrapper.fpas\"]\n",
    );
    write(
        &dir.join("wrapper/wrapper.fpas"),
        "unit Lib.Wrapper;\nuses Lib.Base;\n",
    );
    write(
        &consumer,
        "[project]\nname = \"app\"\nkind = \"program\"\nmain = \"main.fpas\"\n\n[dependencies]\nprojects = [\"../base/base.fpasprj\", \"../wrapper/wrapper.fpasprj\"]\n\n[sources]\ninclude = [\"main.fpas\"]\n",
    );
    write(&dir.join("app/main.fpas"), "program App; begin end.\n");

    let loaded = load_project(&consumer).expect("the same library owner must remain valid");
    fs::remove_dir_all(&dir).ok();

    assert_eq!(loaded.source_files.len(), 2);
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

#[cfg(unix)]
fn create_file_symlink(target: &Path, link: &Path) -> std::io::Result<()> {
    std::os::unix::fs::symlink(target, link)
}

#[cfg(windows)]
fn create_file_symlink(target: &Path, link: &Path) -> std::io::Result<()> {
    std::os::windows::fs::symlink_file(target, link)
}
