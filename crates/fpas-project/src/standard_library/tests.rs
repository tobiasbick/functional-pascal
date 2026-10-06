//! Unit tests for manifest-backed standard library loading.

#![allow(
    clippy::expect_used,
    reason = "unit tests use expect to keep fixture setup compact"
)]

use super::*;
use crate::{
    ProjectLinkMeta, build_unit_graph_for_program_with_standard_library, resolve_program_units,
};
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
        "unit Std.Sample;\nconst\n  Value: integer := 42;\nend unit;\n",
    );
    let program = dir.join("main.fpas");
    write_text(
        &program,
        "program Main;\nuses Std.Sample;\nbegin\n  var Answer: integer := Value;\nend.\n",
    );

    let library = load_standard_library(&dir).expect("standard library must load");
    let graph = build_unit_graph_for_program_with_standard_library(
        &program,
        &[],
        &ProjectLinkMeta::default(),
        &library,
    );
    let linked = graph.and_then(|graph| {
        let source = fs::read_to_string(&program).expect("program source");
        let (program, diagnostics) = fpas_parser::parse(&source);
        assert!(diagnostics.is_empty());
        resolve_program_units(&graph, &program.uses).map(|_| ())
    });
    remove_dir(&dir);

    assert!(linked.is_ok(), "exported source unit must link");
}

#[test]
fn loaded_library_keeps_source_files_authoritative() {
    let dir = temp_dir("cached-unit");
    write_text(
        &dir.join("stdlib.fpasprj"),
        r#"[project]
name = "test-stdlib"
kind = "library"

[sources]
include = ["Std/**/*.fpas"]
"#,
    );
    let unit_path = dir.join("Std/Sample.fpas");
    write_text(
        &unit_path,
        "unit Std.Sample;\nconst\n  Value: integer := 42;\nend unit;\n",
    );
    let program = dir.join("main.fpas");
    write_text(
        &program,
        "program Main;\nuses Std.Sample;\nbegin\n  var Answer: integer := Value;\nend.\n",
    );

    let library = load_standard_library(&dir).expect("standard library must load");
    fs::remove_file(unit_path).expect("cached source file must be removable");
    let linked = build_unit_graph_for_program_with_standard_library(
        &program,
        &[],
        &ProjectLinkMeta::default(),
        &library,
    );
    remove_dir(&dir);

    assert!(
        linked.is_err(),
        "removing an authoritative standard-library source must fail"
    );
}

#[test]
fn editable_standard_library_project_keeps_trusted_source_provenance() {
    let dir = temp_dir("editable-project");
    write_text(
        &dir.join("stdlib.fpasprj"),
        r#"[project]
name = "test-stdlib"
kind = "library"

[sources]
include = ["Std/**/*.fpas"]
"#,
    );
    let unit_path = dir.join("Std/Sample.fpas");
    write_text(&unit_path, "unit Std.Sample;\nend unit;\n");

    let project = load_standard_library_project(&dir).expect("editable standard library must load");
    let origin = project.link_meta.origin_for_source(&unit_path);
    let trusted = project
        .link_meta
        .is_trusted_standard_library_source(&unit_path);
    remove_dir(&dir);

    assert_eq!(origin, SourceOrigin::Own);
    assert!(trusted);
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
    write_text(
        &dir.join("Std/Exported.fpas"),
        "unit Std.Exported;\nend unit;\n",
    );
    write_text(
        &dir.join("Std/Internal.fpas"),
        "unit Std.Internal;\nend unit;\n",
    );
    let program = dir.join("main.fpas");
    write_text(&program, "program Main;\nuses Std.Internal;\nbegin\nend.\n");

    let library = load_standard_library(&dir).expect("standard library must load");
    let result = build_unit_graph_for_program_with_standard_library(
        &program,
        &[],
        &ProjectLinkMeta::default(),
        &library,
    )
    .and_then(|graph| {
        let source = fs::read_to_string(&program).expect("program source");
        let (program, diagnostics) = fpas_parser::parse(&source);
        assert!(diagnostics.is_empty());
        resolve_program_units(&graph, &program.uses).map(|_| ())
    });
    remove_dir(&dir);

    assert!(result.is_err(), "private source unit must be rejected");
    let error = result.expect_err("invalid standard library");
    assert!(
        error.to_string().contains("not exported"),
        "unexpected error: {error}"
    );
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
    write_text(
        &dir.join("Std/Console.fpas"),
        "unit Std.Console;\nend unit;\n",
    );

    let error = load_standard_library(&dir).expect_err("intrinsic collision must fail");
    remove_dir(&dir);

    assert!(
        error.to_string().contains("collides with intrinsic"),
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
    write_text(&dir.join("src/Other.fpas"), "unit Other;\nend unit;\n");

    let error = load_standard_library(&dir).expect_err("non-Std unit must fail");
    remove_dir(&dir);

    assert!(error.to_string().contains("must use the `Std.*` namespace"));
}

#[test]
fn standard_library_requires_manifest() {
    let dir = temp_dir("missing-manifest");

    let error = load_standard_library(&dir).expect_err("missing manifest must fail");
    remove_dir(&dir);

    assert!(error.to_string().contains("stdlib.fpasprj"));
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
