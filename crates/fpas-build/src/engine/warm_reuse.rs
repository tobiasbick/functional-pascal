//! Reusing sidecars must not clone direct dependency interfaces for compilation.
use super::interfaces::DIRECT_INTERFACE_COPIES;
use super::*;
use fpas_project::{build_unit_graph, load_project, resolve_library_units};

#[test]
fn warm_many_import_build_avoids_direct_interface_copies() {
    let root = std::env::temp_dir().join(format!("fpas-warm-imports-{}", std::process::id()));
    std::fs::create_dir_all(&root).expect("fixture directory");
    let manifest = root.join("library.fpasprj");
    std::fs::write(
        &manifest,
        "[project]\nname = \"library\"\nkind = \"library\"\n[sources]\ninclude = [\"*.fpas\"]\n",
    )
    .expect("manifest");
    for id in 0..32 {
        let imports = (0..id)
            .map(|used| format!("U{used}"))
            .collect::<Vec<_>>()
            .join(", ");
        let uses = if imports.is_empty() {
            String::new()
        } else {
            format!("uses {imports};")
        };
        std::fs::write(
            root.join(format!("u{id}.fpas")),
            format!("unit U{id}; {uses} public const Value{id}: integer := {id};"),
        )
        .expect("unit source");
    }
    let project = load_project(&manifest).expect("project");
    let graph = build_unit_graph(&project.source_files, &project.link_meta).expect("graph");
    let selection = resolve_library_units(&graph).expect("selection");
    DIRECT_INTERFACE_COPIES.with(|count| count.set(0));
    let cold =
        build_library_units(&graph, &selection, &BuildOptions::default()).expect("cold build");
    assert_eq!(cold.counters().compiled, 32);
    let cold_copies = DIRECT_INTERFACE_COPIES.with(std::cell::Cell::get);
    assert_eq!(cold_copies, 496);
    DIRECT_INTERFACE_COPIES.with(|count| count.set(0));
    let warm =
        build_library_units(&graph, &selection, &BuildOptions::default()).expect("warm build");
    assert_eq!(warm.counters().sidecar_reused, 32);
    assert_eq!(warm.counters().compiled, 0);
    let warm_copies = DIRECT_INTERFACE_COPIES.with(std::cell::Cell::get);
    println!(
        "units=32, imports=496, cold_direct_copies={cold_copies}, warm_direct_copies={warm_copies}"
    );
    assert_eq!(warm_copies, 0);
    std::fs::remove_dir_all(root).expect("remove fixture");
}
