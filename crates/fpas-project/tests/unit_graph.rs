//! Integration coverage for reusable project unit graphs.
//!
//! Documentation: `docs/pascal/program-structure/units.md` and
//! `docs/pascal/program-structure/projects.md`.

#![allow(
    clippy::expect_used,
    reason = "integration fixtures use expect to keep graph assertions focused"
)]

use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

use fpas_project::{
    SourceOrigin, build_unit_graph, load_project, resolve_library_units, resolve_program_units,
};

fn temp_dir(name: &str) -> PathBuf {
    static NEXT: AtomicU64 = AtomicU64::new(1);
    let id = NEXT.fetch_add(1, Ordering::Relaxed);
    std::env::temp_dir().join(format!(
        "fpas-unit-graph-{name}-{}-{id}",
        std::process::id()
    ))
}

fn write(path: &Path, text: &str) {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).expect("fixture parent must exist");
    }
    fs::write(path, text).expect("fixture must be written");
}

fn write_project(dir: &Path, sources: &[(&str, &str)]) -> PathBuf {
    let manifest = dir.join("demo.fpasprj");
    write(
        &manifest,
        r#"[project]
name = "demo"
kind = "library"

[sources]
include = ["src/**/*.fpas"]
"#,
    );
    for (path, source) in sources {
        write(&dir.join("src").join(path), source);
    }
    manifest
}

fn uses_from_program(source: &str) -> Vec<fpas_parser::QualifiedId> {
    let (program, diagnostics) = fpas_parser::parse(source);
    assert!(
        diagnostics
            .iter()
            .all(|diagnostic| !diagnostic.as_diagnostic().is_error()),
        "program fixture must parse"
    );
    program.uses
}

#[test]
fn graph_records_unit_identity_origin_dependencies_and_source_path() {
    let dir = temp_dir("identity");
    let manifest = write_project(
        &dir,
        &[
            ("core.fpas", "unit Demo.Core;\n"),
            ("feature.fpas", "unit Demo.Feature;\nuses Demo.Core;\n"),
        ],
    );
    let loaded = load_project(&manifest).expect("project must load");

    let graph =
        build_unit_graph(&loaded.source_files, &loaded.link_meta).expect("graph must build");
    let feature = graph.get("demo.feature").expect("feature node");

    assert_eq!(feature.display_name(), "Demo.Feature");
    assert_eq!(feature.canonical_name(), "demo.feature");
    assert!(matches!(feature.origin(), SourceOrigin::Own));
    assert_eq!(feature.direct_uses().len(), 1);
    assert_eq!(
        graph.source_paths()[feature.source_id() as usize],
        feature.path()
    );

    fs::remove_dir_all(dir).ok();
}

#[test]
fn program_resolution_excludes_unreachable_units_and_orders_dependencies_first() {
    let dir = temp_dir("reachable");
    let manifest = write_project(
        &dir,
        &[
            ("base.fpas", "unit Demo.Base;\n"),
            ("feature.fpas", "unit Demo.Feature;\nuses Demo.Base;\n"),
            ("unused.fpas", "unit Demo.Unused;\n"),
        ],
    );
    let loaded = load_project(&manifest).expect("project must load");
    let graph =
        build_unit_graph(&loaded.source_files, &loaded.link_meta).expect("graph must build");
    let root_uses = uses_from_program("program App;\nuses Demo.Feature;\nbegin\nend.\n");

    let resolved = resolve_program_units(&graph, &root_uses).expect("graph must resolve");

    assert_eq!(
        resolved.order(),
        &["demo.base".to_string(), "demo.feature".to_string()]
    );
    assert!(!resolved.order().iter().any(|name| name == "demo.unused"));

    fs::remove_dir_all(dir).ok();
}

#[test]
fn library_resolution_includes_all_units_in_stable_dependency_order() {
    let dir = temp_dir("library-all");
    let manifest = write_project(
        &dir,
        &[
            ("alpha.fpas", "unit Demo.Alpha;\n"),
            ("beta.fpas", "unit Demo.Beta;\nuses Demo.Alpha;\n"),
            ("unused.fpas", "unit Demo.Unused;\n"),
        ],
    );
    let loaded = load_project(&manifest).expect("project must load");
    let graph =
        build_unit_graph(&loaded.source_files, &loaded.link_meta).expect("graph must build");

    let resolved = resolve_library_units(&graph).expect("library graph must resolve");

    assert_eq!(
        resolved.order(),
        &[
            "demo.alpha".to_string(),
            "demo.beta".to_string(),
            "demo.unused".to_string(),
        ]
    );

    fs::remove_dir_all(dir).ok();
}

#[test]
fn graph_resolution_reports_complete_unit_cycle() {
    let dir = temp_dir("cycle");
    let manifest = write_project(
        &dir,
        &[
            ("a.fpas", "unit Demo.A;\nuses Demo.B;\n"),
            ("b.fpas", "unit Demo.B;\nuses Demo.C;\n"),
            ("c.fpas", "unit Demo.C;\nuses Demo.A;\n"),
        ],
    );
    let loaded = load_project(&manifest).expect("project must load");
    let graph =
        build_unit_graph(&loaded.source_files, &loaded.link_meta).expect("graph must build");
    let root_uses = uses_from_program("program App;\nuses Demo.A;\nbegin\nend.\n");

    let error = resolve_program_units(&graph, &root_uses).expect_err("cycle must fail");

    assert!(error.contains("Demo.A -> Demo.B -> Demo.C -> Demo.A"));
    assert!(error.contains("extracting shared declarations"));

    fs::remove_dir_all(dir).ok();
}

#[test]
fn graph_resolution_enforces_dependency_project_unit_exports() {
    let dir = temp_dir("exports");
    let library = dir.join("lib/lib.fpasprj");
    let application = dir.join("app/app.fpasprj");
    write(
        &library,
        r#"[project]
name = "lib"
kind = "library"

[exports]
units = ["Lib.Api"]

[sources]
include = ["src/**/*.fpas"]
"#,
    );
    write(&dir.join("lib/src/api.fpas"), "unit Lib.Api;\n");
    write(&dir.join("lib/src/internal.fpas"), "unit Lib.Internal;\n");
    write(
        &application,
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
        &dir.join("app/src/main.fpas"),
        "program App;\nuses Lib.Internal;\nbegin\nend.\n",
    );
    let loaded = load_project(&application).expect("application project must load");
    let graph =
        build_unit_graph(&loaded.source_files, &loaded.link_meta).expect("graph must build");
    let root_uses = uses_from_program("program App;\nuses Lib.Internal;\nbegin\nend.\n");

    let error =
        resolve_program_units(&graph, &root_uses).expect_err("internal unit must be rejected");

    assert!(error.contains("Lib.Internal"));
    assert!(error.contains("not exported"));
    assert!(error.contains("lib.fpasprj"));

    fs::remove_dir_all(dir).ok();
}

#[test]
fn unknown_transitive_unit_diagnostic_names_owner_and_known_units() {
    let dir = temp_dir("unknown");
    let manifest = write_project(
        &dir,
        &[("feature.fpas", "unit Demo.Feature;\nuses Demo.Missing;\n")],
    );
    let loaded = load_project(&manifest).expect("project must load");
    let graph =
        build_unit_graph(&loaded.source_files, &loaded.link_meta).expect("graph must build");
    let root_uses = uses_from_program("program App;\nuses Demo.Feature;\nbegin\nend.\n");

    let error = resolve_program_units(&graph, &root_uses).expect_err("missing unit must fail");

    assert!(error.contains("Demo.Missing"));
    assert!(error.contains("unit `Demo.Feature`"));
    assert!(error.contains("Available units: Demo.Feature"));

    fs::remove_dir_all(dir).ok();
}
