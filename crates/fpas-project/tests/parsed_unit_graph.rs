//! Regression coverage for overlay-safe parsed-source unit graphs.

#![allow(
    clippy::expect_used,
    reason = "integration fixtures use expect to keep graph assertions focused"
)]
#![expect(
    clippy::panic,
    reason = "fixture helper panics only when its hard-coded source has the wrong compilation-unit shape"
)]

use std::path::{Path, PathBuf};

use fpas_parser::{CompilationUnit, parse_compilation_unit};
use fpas_project::{
    ProjectLinkMeta, build_unit_graph_for_program_from_parsed_sources,
    build_unit_graph_from_parsed_sources, resolve_program_units,
};

fn parsed_unit(path: &str, source: &str) -> (PathBuf, fpas_parser::Unit) {
    let (parsed, diagnostics) = parse_compilation_unit(source);
    assert!(diagnostics.is_empty(), "{diagnostics:?}");
    let CompilationUnit::Unit(unit) = parsed else {
        panic!("fixture must declare a unit");
    };
    (PathBuf::from(path), unit)
}

#[test]
fn parsed_source_graph_does_not_read_nonexistent_source_paths() {
    let sources = vec![
        parsed_unit("virtual/base.fpas", "unit Demo.Base;\n"),
        parsed_unit(
            "virtual/feature.fpas",
            "unit Demo.Feature;\nuses Demo.Base;\n",
        ),
    ];

    let graph = build_unit_graph_from_parsed_sources(sources, &ProjectLinkMeta::default())
        .expect("in-memory graph");

    assert_eq!(graph.len(), 2);
    assert_eq!(graph.source_paths()[0], Path::new("virtual/base.fpas"));
    assert_eq!(graph.source_paths()[1], Path::new("virtual/feature.fpas"));
}

#[test]
fn parsed_program_graph_reserves_main_source_and_resolves_dependencies() {
    let sources = vec![
        parsed_unit("virtual/base.fpas", "unit Demo.Base;\n"),
        parsed_unit(
            "virtual/feature.fpas",
            "unit Demo.Feature;\nuses Demo.Base;\n",
        ),
    ];
    let (program, diagnostics) =
        fpas_parser::parse("program App;\nuses Demo.Feature;\nbegin\nend.\n");
    assert!(diagnostics.is_empty(), "{diagnostics:?}");

    let graph = build_unit_graph_for_program_from_parsed_sources(
        Path::new("virtual/main.fpas"),
        sources,
        &ProjectLinkMeta::default(),
    )
    .expect("in-memory program graph");
    let resolved = resolve_program_units(&graph, &program.uses).expect("resolved graph");

    assert_eq!(graph.source_paths()[0], Path::new("virtual/main.fpas"));
    assert_eq!(
        resolved.order(),
        &["demo.base".to_string(), "demo.feature".to_string()]
    );
}

#[test]
fn parsed_graph_accepts_reserved_std_names_from_trusted_library_sources() {
    let path = PathBuf::from("virtual/Std/Sample.fpas");
    let sources = vec![parsed_unit(
        path.to_str().expect("UTF-8 fixture path"),
        "unit Std.Sample;\n",
    )];
    let mut link_meta = ProjectLinkMeta::default();
    link_meta.trusted_standard_library_sources.insert(path);

    let graph = build_unit_graph_from_parsed_sources(sources, &link_meta)
        .expect("trusted standard-library graph");

    assert_eq!(graph.len(), 1);
}

#[test]
fn parsed_graph_still_validates_names_from_regular_library_dependencies() {
    let path = PathBuf::from("virtual/Std/Sample.fpas");
    let sources = vec![parsed_unit(
        path.to_str().expect("UTF-8 fixture path"),
        "unit Std.Sample;\n",
    )];
    let mut link_meta = ProjectLinkMeta::default();
    link_meta.source_origins.insert(
        path,
        fpas_project::SourceOrigin::Library(PathBuf::from("virtual/dependency.fpasprj")),
    );

    let error = build_unit_graph_from_parsed_sources(sources, &link_meta)
        .expect_err("regular dependencies must not claim the Std namespace");

    assert!(error.contains("reserved for standard library units"));
}

#[test]
fn parsed_graph_rejects_reserved_std_names_from_user_sources() {
    let sources = vec![parsed_unit("virtual/Std/Sample.fpas", "unit Std.Sample;\n")];

    let error = build_unit_graph_from_parsed_sources(sources, &ProjectLinkMeta::default())
        .expect_err("user project must not claim the Std namespace");

    assert!(error.contains("reserved for standard library units"));
}
