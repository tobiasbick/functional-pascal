//! Public build/check APIs preserve diagnostics until the caller renders them.

#![allow(
    clippy::expect_used,
    reason = "filesystem fixtures use expectations to identify failing setup steps"
)]

use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

use fpas_build::{
    BuildOptions, ProgramArtifactTarget, build_library_units, build_program,
    build_program_artifact, check_library_units, check_program,
};
use fpas_project::{
    ProjectLinkMeta, build_unit_graph_for_program, resolve_library_units, resolve_program_units,
};

struct Fixture(PathBuf);

impl Fixture {
    fn new() -> Self {
        static NEXT: AtomicU64 = AtomicU64::new(0);
        let id = NEXT.fetch_add(1, Ordering::Relaxed);
        let path = std::env::temp_dir().join(format!(
            "fpas-build-diagnostics-{}-{id}",
            std::process::id()
        ));
        std::fs::create_dir_all(&path).expect("fixture directory");
        Self(path)
    }

    fn write(&self, name: &str, source: &str) -> PathBuf {
        let path = self.0.join(name);
        std::fs::write(&path, source).expect("fixture source");
        path
    }
}

impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

#[test]
fn root_build_and_check_preserve_multiple_compiler_diagnostics() {
    let source = "program Demo; begin const A: integer := 'x'; const B: boolean := 42; end.";
    let (program, parse_errors) = fpas_parser::parse(source);
    assert!(parse_errors.is_empty());
    let expected = fpas_compiler::compile_program_object_with_support(&program, &[], &[])
        .expect_err("type errors");
    assert!(expected.len() >= 2);
    let graph =
        build_unit_graph_for_program(Path::new("main.fpas"), &[], &ProjectLinkMeta::default())
            .expect("empty graph");
    let selection = resolve_program_units(&graph, &program.uses).expect("selection");
    for result in [
        build_program(&graph, &selection, &program, &BuildOptions::default()),
        check_program(&graph, &selection, &program, &BuildOptions::default()),
    ] {
        let error = result.err().expect("invalid root program");
        let actual: Vec<_> = error
            .diagnostics()
            .iter()
            .map(|record| &record.diagnostic)
            .collect();
        assert_eq!(actual, expected.iter().collect::<Vec<_>>());
        assert!(
            error
                .diagnostics()
                .iter()
                .all(|record| record.path.is_none())
        );
        assert_eq!(
            error.to_string(),
            expected
                .iter()
                .map(fpas_diagnostics::render_without_path)
                .collect::<Vec<_>>()
                .join("\n")
        );
    }
}

#[test]
fn imported_unit_diagnostics_keep_the_unit_path_and_nonzero_source_id() {
    let fixture = Fixture::new();
    let unit = fixture.write(
        "broken.fpas",
        "unit Broken; public const Number: integer := 'bad';\nend unit;",
    );
    let graph = build_unit_graph_for_program(
        &fixture.0.join("main.fpas"),
        std::slice::from_ref(&unit),
        &ProjectLinkMeta::default(),
    )
    .expect("parsed unit graph");
    let node = graph.get("broken").expect("unit node");
    assert_ne!(node.source_id(), 0);
    let selection = resolve_library_units(&graph).expect("library selection");
    for result in [
        build_library_units(&graph, &selection, &BuildOptions::default()),
        check_library_units(&graph, &selection, &BuildOptions::default()),
    ] {
        let error = result.err().expect("invalid imported unit");
        assert!(!error.diagnostics().is_empty());
        for record in error.diagnostics() {
            assert_eq!(record.path.as_deref(), Some(unit.as_path()));
            assert_eq!(
                record.diagnostic.span.expect("source span").source_id(),
                node.source_id()
            );
            assert_eq!(
                record.diagnostic.stage(),
                fpas_diagnostics::DiagnosticStage::Sema
            );
        }
        assert!(
            error
                .to_string()
                .starts_with(&unit.to_string_lossy().to_string())
        );
    }
    assert!(!unit.with_extension("fpascu").exists());
}

#[test]
fn artifact_parser_retains_every_diagnostic_and_expectation_detail() {
    let source = "program Demo\nbegin\n var X := ;\nend.";
    let (_, expected) = fpas_parser::parse_compilation_unit(source);
    assert!(expected.len() >= 2);
    assert!(
        expected
            .iter()
            .any(|error| error.as_diagnostic().expected.is_some())
    );
    let graph =
        build_unit_graph_for_program(Path::new("main.fpas"), &[], &ProjectLinkMeta::default())
            .expect("empty graph");
    let source_paths = vec!["main.fpas".to_owned()];
    let error = build_program_artifact(
        &graph,
        ProgramArtifactTarget {
            path: Path::new("unused.fpascp"),
            source: source.as_bytes(),
            source_paths: &source_paths,
        },
        &BuildOptions::default(),
    )
    .err()
    .expect("invalid program source");
    let actual: Vec<_> = error
        .diagnostics()
        .iter()
        .map(|record| &record.diagnostic)
        .collect();
    assert_eq!(
        actual,
        expected
            .iter()
            .map(fpas_parser::ParseDiagnostic::as_diagnostic)
            .collect::<Vec<_>>()
    );
    assert!(
        error
            .diagnostics()
            .iter()
            .all(|record| record.path.as_deref() == Some(Path::new("main.fpas")))
    );
}

#[test]
fn artifact_compiler_errors_keep_the_supplied_main_path() {
    let fixture = Fixture::new();
    let source = "program Demo; begin const X: integer := 'bad'; end.";
    let main = fixture.write("main.fpas", source);
    let graph =
        build_unit_graph_for_program(&main, &[], &ProjectLinkMeta::default()).expect("graph");
    let paths = vec!["src/main.fpas".to_owned()];
    let artifact = fixture.0.join("main.fpascp");
    let error = build_program_artifact(
        &graph,
        ProgramArtifactTarget {
            path: &artifact,
            source: source.as_bytes(),
            source_paths: &paths,
        },
        &BuildOptions::default(),
    )
    .err()
    .expect("semantic error");
    assert!(!error.diagnostics().is_empty());
    assert!(
        error
            .diagnostics()
            .iter()
            .all(|record| record.path.as_deref() == Some(Path::new("src/main.fpas")))
    );
    assert!(!artifact.exists());
    let expected = error
        .diagnostics()
        .iter()
        .map(|record| fpas_diagnostics::render_without_path(&record.diagnostic))
        .collect::<Vec<_>>()
        .join("\n");
    assert_eq!(error.to_string(), expected);
}

#[test]
fn project_read_errors_keep_their_path_without_a_source_span_in_build_errors() {
    let fixture = Fixture::new();
    let missing = fixture.0.join("missing.fpas");
    let project_error =
        fpas_project::build_unit_graph(std::slice::from_ref(&missing), &ProjectLinkMeta::default())
            .expect_err("missing source");
    let expected_text = project_error.to_string();
    let error = fpas_build::BuildError::from(project_error);
    assert_eq!(error.diagnostics().len(), 1);
    let record = &error.diagnostics()[0];
    assert_eq!(record.path.as_deref(), Some(missing.as_path()));
    assert_eq!(record.diagnostic.span, None);
    assert_eq!(
        record.diagnostic.code,
        fpas_diagnostics::codes::PROJECT_SOURCE_READ_FAILED
    );
    assert_eq!(error.to_string(), expected_text);
}

#[test]
fn snapshot_parser_records_survive_conversion_to_build_errors() {
    let fixture = Fixture::new();
    let unit = fixture.write("demo.fpas", "unit Demo;\nend unit;");
    let graph = build_unit_graph_for_program(
        &fixture.0.join("main.fpas"),
        std::slice::from_ref(&unit),
        &ProjectLinkMeta::default(),
    )
    .expect("valid graph");
    let node = graph.get("demo").expect("unit node");
    let project_error = node
        .parse_source_snapshot(b"unit Demo\npublic const X: integer := ;")
        .expect_err("invalid snapshot");
    let expected = project_error.diagnostics().to_vec();
    assert!(expected.len() > 1);
    let error = fpas_build::BuildError::from(project_error);
    assert_eq!(error.diagnostics().len(), expected.len());
    for (record, original) in error.diagnostics().iter().zip(expected) {
        assert_eq!(record.diagnostic, original);
        assert_eq!(record.path.as_deref(), Some(unit.as_path()));
        assert_eq!(
            record.diagnostic.span.expect("source span").source_id(),
            node.source_id()
        );
    }
}
