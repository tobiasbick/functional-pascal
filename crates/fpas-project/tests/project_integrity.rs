//! Regressions for source-authoritative project and Unit graph identities.

#![allow(
    clippy::expect_used,
    reason = "integration fixtures use expect to keep assertions focused"
)]
#![expect(
    clippy::panic,
    reason = "the parser fixture panics only when its hard-coded source has the wrong shape"
)]

use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::atomic::{AtomicU64, Ordering};

use fpas_parser::{CompilationUnit, QualifiedId, parse_compilation_unit};
use fpas_project::{
    LibraryExportPolicy, ProjectLinkMeta, SourceOrigin, build_unit_graph,
    build_unit_graph_from_parsed_sources, build_unit_graph_with_standard_library, load_project,
    load_standard_library, resolve_program_units,
};
use fpas_unit::{CompiledUnit, DependencyIdentity, Digest, UnitIdentity};

fn temp_dir(name: &str) -> PathBuf {
    static NEXT: AtomicU64 = AtomicU64::new(1);
    let id = NEXT.fetch_add(1, Ordering::Relaxed);
    std::env::temp_dir().join(format!(
        "fpas-project-integrity-{name}-{}-{id}",
        std::process::id()
    ))
}

fn write(path: &Path, text: &str) {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).expect("fixture directory");
    }
    fs::write(path, text).expect("fixture file");
}

fn uses(source: &str) -> Vec<QualifiedId> {
    let (parsed, diagnostics) = fpas_parser::parse(source);
    assert!(diagnostics.is_empty(), "{diagnostics:?}");
    parsed.uses
}

fn parsed_unit(source: &str) -> fpas_parser::Unit {
    let (parsed, diagnostics) = parse_compilation_unit(source);
    assert!(diagnostics.is_empty(), "{diagnostics:?}");
    let CompilationUnit::Unit(unit) = parsed else {
        panic!("fixture must declare a Unit")
    };
    unit
}

#[test]
fn unknown_root_std_unit_is_rejected() {
    let graph = build_unit_graph(&[], &ProjectLinkMeta::default()).expect("empty graph");
    let root_uses = uses("program App;\nuses Std.DoesNotExist;\nbegin\nend.\n");

    let error = resolve_program_units(&graph, &root_uses).expect_err("unknown Unit must fail");

    assert!(error.contains("Unknown unit `Std.Doesnotexist`"), "{error}");
}

#[test]
fn unknown_transitive_std_unit_is_rejected() {
    let dir = temp_dir("transitive-std");
    let source = dir.join("feature.fpas");
    write(&source, "unit Demo.Feature;\nuses Std.DoesNotExist;\n");
    let graph = build_unit_graph(&[source], &ProjectLinkMeta::default()).expect("graph");
    let root_uses = uses("program App;\nuses Demo.Feature;\nbegin\nend.\n");

    let error = resolve_program_units(&graph, &root_uses).expect_err("unknown Unit must fail");

    assert!(error.contains("unit `Demo.Feature`"), "{error}");
    fs::remove_dir_all(dir).ok();
}

#[test]
fn known_intrinsic_std_unit_needs_no_source_node() {
    let graph = build_unit_graph(&[], &ProjectLinkMeta::default()).expect("empty graph");
    let root_uses = uses("program App;\nuses Std.Console;\nbegin\nend.\n");

    let resolved = resolve_program_units(&graph, &root_uses).expect("intrinsic Unit");

    assert!(resolved.is_empty());
}

#[test]
fn source_defined_std_tui_must_be_present() {
    let graph = build_unit_graph(&[], &ProjectLinkMeta::default()).expect("empty graph");
    let root_uses = uses("program App;\nuses Std.Tui;\nbegin\nend.\n");

    let error = resolve_program_units(&graph, &root_uses).expect_err("missing source Unit");

    assert!(error.contains("Unknown unit `Std.Tui`"), "{error}");
}

#[test]
fn source_defined_std_tui_resolves_from_standard_library() {
    let dir = temp_dir("source-std");
    write(
        &dir.join("stdlib.fpasprj"),
        "[project]\nname = \"stdlib\"\nkind = \"library\"\n\n[exports]\nunits = [\"Std.Tui\"]\n\n[sources]\ninclude = [\"Std/**/*.fpas\"]\n",
    );
    write(&dir.join("Std/Tui.fpas"), "unit Std.Tui;\n");
    let standard_library = load_standard_library(&dir).expect("standard library");
    let graph =
        build_unit_graph_with_standard_library(&[], &ProjectLinkMeta::default(), &standard_library)
            .expect("graph");
    let root_uses = uses("program App;\nuses Std.Tui;\nbegin\nend.\n");

    let resolved = resolve_program_units(&graph, &root_uses).expect("source Unit");

    assert_eq!(resolved.order(), &["std.tui".to_string()]);
    fs::remove_dir_all(dir).ok();
}

#[test]
fn graph_dependencies_come_from_source_instead_of_matching_hash_sidecar() {
    let dir = temp_dir("sidecar");
    let feature = dir.join("feature.fpas");
    let real = dir.join("real.fpas");
    let source = b"unit Demo.Feature;\nuses Demo.Real;\n";
    fs::create_dir_all(&dir).expect("fixture directory");
    fs::write(&feature, source).expect("feature source");
    write(&real, "unit Demo.Real;\n");
    let stale = CompiledUnit {
        identity: UnitIdentity {
            unit_name: "Demo.StaleOwner".to_string(),
            source_hash: Digest::of(source),
            interface_hash: Digest::of([]),
            object_hash: Digest::of([]),
            compiler_version: "stale-compiler".to_string(),
            bytecode_version: u32::MAX,
            options_hash: Digest::of(b"stale-options"),
            dependencies: vec![DependencyIdentity {
                unit_name: "demo.stale".to_string(),
                interface_hash: Digest::of(b"stale-interface"),
            }],
        },
        interface: Vec::new(),
        object: Vec::new(),
    };
    let bytes = fpas_unit::encode(&stale).expect("encodable stale sidecar");
    fs::write(fpas_unit::sidecar_path(&feature), bytes).expect("sidecar fixture");

    let graph = build_unit_graph(&[feature, real], &ProjectLinkMeta::default()).expect("graph");
    let node = graph.get("demo.feature").expect("source Unit name");

    assert_eq!(node.direct_uses()[0].parts.join("."), "Demo.Real");
    fs::remove_dir_all(dir).ok();
}

#[test]
fn lexical_aliases_preserve_library_exports() {
    let dir = temp_dir("alias-export");
    let source = dir.join("src/internal.fpas");
    let library = dir.join("lib/library.fpasprj");
    write(&source, "unit Lib.Internal;\n");
    write(&library, "fixture");
    let source_alias = dir.join("src/../src/internal.fpas");
    let library_alias = dir.join("lib/../lib/library.fpasprj");
    let mut link_meta = ProjectLinkMeta::default();
    link_meta
        .source_origins
        .insert(source_alias, SourceOrigin::Library(library.clone()));
    link_meta.library_export_policies.insert(
        library_alias,
        LibraryExportPolicy::ListedUnits(HashSet::new()),
    );
    let graph = build_unit_graph(&[source], &link_meta).expect("graph");
    let root_uses = uses("program App;\nuses Lib.Internal;\nbegin\nend.\n");

    let error = resolve_program_units(&graph, &root_uses).expect_err("private Unit must fail");

    assert!(error.contains("not exported"), "{error}");
    fs::remove_dir_all(dir).ok();
}

#[test]
fn trusted_std_source_lookup_accepts_lexical_alias() {
    let dir = temp_dir("alias-std");
    let source = dir.join("Std/Source.fpas");
    write(&source, "unit Std.Source;\n");
    let canonical = fs::canonicalize(&source).expect("canonical source");
    let mut link_meta = ProjectLinkMeta::default();
    link_meta
        .trusted_standard_library_sources
        .insert(dir.join("Std/../Std/Source.fpas"));

    let graph = build_unit_graph_from_parsed_sources(
        vec![(canonical, parsed_unit("unit Std.Source;\n"))],
        &link_meta,
    )
    .expect("trusted parsed graph");

    assert!(graph.contains("std.source"));
    fs::remove_dir_all(dir).ok();
}

#[test]
fn symlink_alias_preserves_library_origin() {
    let dir = temp_dir("symlink-origin");
    let source = dir.join("src/internal.fpas");
    let alias = dir.join("alias.fpas");
    let library = dir.join("library.fpasprj");
    write(&source, "unit Lib.Internal;\n");
    write(&library, "fixture");
    if create_file_symlink(&source, &alias).is_err() {
        fs::remove_dir_all(dir).ok();
        return;
    }
    let mut link_meta = ProjectLinkMeta::default();
    link_meta
        .source_origins
        .insert(alias, SourceOrigin::Library(library.clone()));
    link_meta
        .library_export_policies
        .insert(library, LibraryExportPolicy::ListedUnits(HashSet::new()));
    let graph = build_unit_graph(&[source], &link_meta).expect("graph");
    let root_uses = uses("program App;\nuses Lib.Internal;\nbegin\nend.\n");

    let error = resolve_program_units(&graph, &root_uses).expect_err("private Unit must fail");

    assert!(error.contains("not exported"), "{error}");
    fs::remove_dir_all(dir).ok();
}

#[cfg(unix)]
fn create_file_symlink(target: &Path, link: &Path) -> std::io::Result<()> {
    std::os::unix::fs::symlink(target, link)
}

#[cfg(windows)]
fn create_file_symlink(target: &Path, link: &Path) -> std::io::Result<()> {
    std::os::windows::fs::symlink_file(target, link)
}

#[test]
fn load_project_by_basename_resolves_enclosing_workspace() {
    const CHILD_ROOT: &str = "FPAS_PROJECT_BASENAME_FIXTURE";
    if std::env::var_os(CHILD_ROOT).is_some() {
        let loaded = load_project(Path::new("app.fpasprj")).expect("basename project load");
        assert_eq!(loaded.source_files.len(), 1);
        assert!(matches!(
            loaded.link_meta.origin_for_source(&loaded.source_files[0]),
            SourceOrigin::Library(_)
        ));
        return;
    }

    let dir = temp_dir("basename");
    write(
        &dir.join("suite.fpasworkspace"),
        "[workspace]\nname = \"suite\"\nmembers = [\"app.fpasprj\", \"lib.fpasprj\"]\n",
    );
    write(
        &dir.join("app.fpasprj"),
        "[project]\nname = \"app\"\nkind = \"program\"\nmain = \"main.fpas\"\n\n[dependencies]\nworkspace = [\"lib\"]\n\n[sources]\ninclude = [\"main.fpas\"]\n",
    );
    write(&dir.join("main.fpas"), "program App;\nbegin\nend.\n");
    write(
        &dir.join("lib.fpasprj"),
        "[project]\nname = \"lib\"\nkind = \"library\"\n\n[sources]\ninclude = [\"lib.fpas\"]\n",
    );
    write(&dir.join("lib.fpas"), "unit Lib.Core;\n");

    let status = Command::new(std::env::current_exe().expect("test executable"))
        .arg("--exact")
        .arg("load_project_by_basename_resolves_enclosing_workspace")
        .arg("--nocapture")
        .env(CHILD_ROOT, "1")
        .current_dir(&dir)
        .status()
        .expect("child test process");

    assert!(status.success(), "basename child test failed: {status}");
    fs::remove_dir_all(dir).ok();
}
