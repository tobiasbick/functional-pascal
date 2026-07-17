//! Unit tests for manifest-backed standard library loading.

#![allow(
    clippy::expect_used,
    reason = "unit tests use expect to keep fixture setup compact"
)]

use super::*;
use crate::{ProjectLinkMeta, build_program_with_standard_library};
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

#[test]
fn manifest_backed_library_links_an_exported_std_unit() {
    let dir = temp_dir("exported-unit");
    write_text(
        &dir.join("stdlib.fpasprj"),
        r#"[project]
name = "test-stdlib"
kind = "library"

[exports]
units = ["Std.Sample"]

[sources]
include = ["Std/**/*.fpas"]
"#,
    );
    write_text(
        &dir.join("Std/Sample.fpas"),
        "unit Std.Sample;\nconst\n  Value: integer := 42;\n",
    );
    let program = dir.join("main.fpas");
    write_text(
        &program,
        "program Main;\nuses Std.Sample;\nbegin\n  var Answer: integer := Value\nend.\n",
    );

    let library = load_standard_library(&dir).expect("standard library must load");
    let linked =
        build_program_with_standard_library(&program, &[], &ProjectLinkMeta::default(), &library);
    remove_dir(&dir);

    assert!(linked.is_ok(), "exported source unit must link");
}

#[test]
fn private_standard_library_unit_is_not_importable_by_programs() {
    let dir = temp_dir("private-unit");
    write_text(
        &dir.join("stdlib.fpasprj"),
        r#"[project]
name = "test-stdlib"
kind = "library"

[exports]
units = ["Std.Exported"]

[sources]
include = ["Std/**/*.fpas"]
"#,
    );
    write_text(&dir.join("Std/Exported.fpas"), "unit Std.Exported;\n");
    write_text(&dir.join("Std/Internal.fpas"), "unit Std.Internal;\n");
    let program = dir.join("main.fpas");
    write_text(&program, "program Main;\nuses Std.Internal;\nbegin\nend.\n");

    let library = load_standard_library(&dir).expect("standard library must load");
    let result =
        build_program_with_standard_library(&program, &[], &ProjectLinkMeta::default(), &library);
    remove_dir(&dir);

    assert!(result.is_err(), "private source unit must be rejected");
    let error = result.err().unwrap_or_default();
    assert!(error.contains("not exported"), "unexpected error: {error}");
}

#[test]
fn standard_library_rejects_intrinsic_unit_collision() {
    let dir = temp_dir("intrinsic-collision");
    write_text(
        &dir.join("stdlib.fpasprj"),
        r#"[project]
name = "test-stdlib"
kind = "library"

[sources]
include = ["Std/**/*.fpas"]
"#,
    );
    write_text(&dir.join("Std/Console.fpas"), "unit Std.Console;\n");

    let error = load_standard_library(&dir).expect_err("intrinsic collision must fail");
    remove_dir(&dir);

    assert!(
        error.contains("collides with intrinsic"),
        "unexpected error: {error}"
    );
}

#[test]
fn standard_library_rejects_sources_outside_std_namespace() {
    let dir = temp_dir("invalid-namespace");
    write_text(
        &dir.join("stdlib.fpasprj"),
        r#"[project]
name = "test-stdlib"
kind = "library"

[sources]
include = ["src/**/*.fpas"]
"#,
    );
    write_text(&dir.join("src/Other.fpas"), "unit Other;\n");

    let error = load_standard_library(&dir).expect_err("non-Std unit must fail");
    remove_dir(&dir);

    assert!(error.contains("must use the `Std.*` namespace"));
}

#[test]
fn standard_library_requires_manifest() {
    let dir = temp_dir("missing-manifest");

    let error = load_standard_library(&dir).expect_err("missing manifest must fail");
    remove_dir(&dir);

    assert!(error.contains("stdlib.fpasprj"));
}

fn temp_dir(name: &str) -> PathBuf {
    static NEXT_ID: AtomicU64 = AtomicU64::new(1);
    let id = NEXT_ID.fetch_add(1, Ordering::Relaxed);
    let path = std::env::temp_dir().join(format!("fpas-standard-library-{name}-{id}"));
    fs::create_dir_all(&path).expect("test directory must be created");
    path
}

fn write_text(path: &Path, text: &str) {
    let parent = path.parent().expect("test path must have a parent");
    fs::create_dir_all(parent).expect("test parent must be created");
    fs::write(path, text).expect("test file must be written");
}

fn remove_dir(path: &Path) {
    fs::remove_dir_all(path).expect("test directory must be removed");
}
