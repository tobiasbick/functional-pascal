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

fn uses_from_program(source: &str) -> Vec<fpas_parser::Import> {
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
            (
                "core.fpas",
                r#"unit Demo.Core;
end unit;

"#,
            ),
            (
                "feature.fpas",
                r#"unit Demo.Feature;
uses Demo.Core as Core;
end unit;

"#,
            ),
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
            (
                "base.fpas",
                r#"unit Demo.Base;
end unit;

"#,
            ),
            (
                "feature.fpas",
                r#"unit Demo.Feature;
uses Demo.Base as Base;
end unit;

"#,
            ),
            (
                "unused.fpas",
                r#"unit Demo.Unused;
end unit;

"#,
            ),
        ],
    );
    let loaded = load_project(&manifest).expect("project must load");
    let graph =
        build_unit_graph(&loaded.source_files, &loaded.link_meta).expect("graph must build");
    let root_uses = uses_from_program(
        r#"program App;
uses Demo.Feature as Feature;
begin null;
end program;
"#,
    );

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
            (
                "alpha.fpas",
                r#"unit Demo.Alpha;
end unit;

"#,
            ),
            (
                "beta.fpas",
                r#"unit Demo.Beta;
uses Demo.Alpha as Alpha;
end unit;

"#,
            ),
            (
                "unused.fpas",
                r#"unit Demo.Unused;
end unit;

"#,
            ),
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
            (
                "a.fpas",
                r#"unit Demo.A;
uses Demo.B as B;
end unit;

"#,
            ),
            (
                "b.fpas",
                r#"unit Demo.B;
uses Demo.C as C;
end unit;

"#,
            ),
            (
                "c.fpas",
                r#"unit Demo.C;
uses Demo.A as A;
end unit;

"#,
            ),
        ],
    );
    let loaded = load_project(&manifest).expect("project must load");
    let graph =
        build_unit_graph(&loaded.source_files, &loaded.link_meta).expect("graph must build");
    let root_uses = uses_from_program(
        r#"program App;
uses Demo.A as A;
begin null;
end program;
"#,
    );

    let error = resolve_program_units(&graph, &root_uses).expect_err("cycle must fail");

    assert!(
        error
            .to_string()
            .contains("Demo.A -> Demo.B -> Demo.C -> Demo.A")
    );
    assert!(error.to_string().contains("extracting shared declarations"));

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
    write(
        &dir.join("lib/src/api.fpas"),
        r#"unit Lib.Api;
end unit;

"#,
    );
    write(
        &dir.join("lib/src/internal.fpas"),
        r#"unit Lib.Internal;
end unit;

"#,
    );
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
        r#"program App;
uses Lib.Internal as Internal;
begin null;
end program;
"#,
    );
    let loaded = load_project(&application).expect("application project must load");
    let graph =
        build_unit_graph(&loaded.source_files, &loaded.link_meta).expect("graph must build");
    let root_uses = uses_from_program(
        r#"program App;
uses Lib.Internal as Internal;
begin null;
end program;
"#,
    );

    let error =
        resolve_program_units(&graph, &root_uses).expect_err("internal unit must be rejected");

    assert!(error.to_string().contains("Lib.Internal"));
    assert!(error.to_string().contains("not exported"));
    assert!(error.to_string().contains("lib.fpasprj"));

    fs::remove_dir_all(dir).ok();
}

#[test]
fn unknown_transitive_unit_diagnostic_names_owner_and_known_units() {
    let dir = temp_dir("unknown");
    let manifest = write_project(
        &dir,
        &[(
            "feature.fpas",
            r#"unit Demo.Feature;
uses Demo.Missing as Missing;
end unit;

"#,
        )],
    );
    let loaded = load_project(&manifest).expect("project must load");
    let graph =
        build_unit_graph(&loaded.source_files, &loaded.link_meta).expect("graph must build");
    let root_uses = uses_from_program(
        r#"program App;
uses Demo.Feature as Feature;
begin null;
end program;
"#,
    );

    let error = resolve_program_units(&graph, &root_uses).expect_err("missing unit must fail");

    assert!(error.to_string().contains("Demo.Missing"));
    assert!(error.to_string().contains("unit `Demo.Feature`"));
    assert!(
        error
            .to_string()
            .contains("Known units in `Demo`: Demo.Feature")
    );

    fs::remove_dir_all(dir).ok();
}
