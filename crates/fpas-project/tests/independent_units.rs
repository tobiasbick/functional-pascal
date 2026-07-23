//! Independent unit analysis driven by the reusable project graph.

#![allow(
    clippy::expect_used,
    reason = "integration fixtures use expect to keep semantic assertions focused"
)]

use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

use fpas_project::{build_unit_graph, load_project, resolve_library_units};
use fpas_unit::interface::UnitInterface;

fn temp_dir() -> PathBuf {
    static NEXT: AtomicU64 = AtomicU64::new(1);
    let id = NEXT.fetch_add(1, Ordering::Relaxed);
    std::env::temp_dir().join(format!(
        "fpas-independent-units-{}-{id}",
        std::process::id()
    ))
}

fn write(path: &Path, source: &str) {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).expect("fixture directory");
    }
    fs::write(path, source).expect("fixture source");
}

#[test]
fn graph_order_analyzes_each_unit_from_direct_interfaces_only() {
    let root = temp_dir();
    let manifest = root.join("demo.fpasprj");
    write(
        &manifest,
        r#"[project]
name = "demo"
kind = "library"

[sources]
include = ["src/**/*.fpas"]
"#,
    );
    write(
        &root.join("src/base.fpas"),
        "unit Demo.Base;
         type Point = record X: integer := 1; end;
         function Make(X: integer): Point;
         begin return record X := X; end end;",
    );
    write(
        &root.join("src/math.fpas"),
        "unit Demo.Math;
         uses Demo.Base;
         function Compute(X: integer): integer;
         begin
           var P: Point := Make(X);
           return P.X
         end;",
    );
    write(
        &root.join("src/app.fpas"),
        "unit Demo.App;
         uses Demo.Math;
         function Run(): integer;
         begin
           return Compute(7)
         end;",
    );

    let project = load_project(&manifest).expect("project load");
    let graph = build_unit_graph(&project.source_files, &project.link_meta).expect("unit graph");
    let resolved = resolve_library_units(&graph).expect("library order");
    let mut interfaces = HashMap::<String, UnitInterface>::new();

    for unit_name in resolved.order() {
        let node = graph.get(unit_name).expect("resolved node");
        let direct_interfaces: Vec<UnitInterface> = node
            .direct_uses()
            .iter()
            .filter_map(|dependency| {
                interfaces
                    .get(&dependency.parts.join(".").to_ascii_lowercase())
                    .cloned()
            })
            .collect();
        let analysis = fpas_sema::analyze_unit(
            node.parsed_unit().expect("unit must parse"),
            &direct_interfaces,
        )
        .expect("interface installation");
        assert!(
            analysis.metadata.0.is_empty(),
            "{} diagnostics: {:#?}",
            node.display_name(),
            analysis.metadata.0
        );
        interfaces.insert(
            unit_name.clone(),
            analysis.interface.expect("valid unit interface"),
        );
    }

    assert_eq!(interfaces.len(), 3);
    fs::remove_dir_all(root).ok();
}
