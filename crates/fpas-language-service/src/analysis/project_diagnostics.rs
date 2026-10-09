//! Original project records survive analysis, discovery and standard-library loading.
//!
//! **Documentation:** `docs/pascal/tools/diagnostics.md`

use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

use fpas_diagnostics::FileDiagnostic;
use fpas_parser::CompilationUnit;
use fpas_project::ProjectError;

use crate::LanguageService;

struct Fixture(PathBuf);

impl Fixture {
    fn new(label: &str) -> Self {
        static NEXT: AtomicU64 = AtomicU64::new(0);
        let path = std::env::temp_dir().join(format!(
            "fpas-project-records-{label}-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        std::fs::create_dir_all(&path).expect("fixture directory");
        Self(path)
    }

    fn write(&self, name: &str, text: &str) -> PathBuf {
        let path = self.0.join(name);
        std::fs::create_dir_all(path.parent().expect("parent")).expect("fixture parents");
        std::fs::write(&path, text).expect("fixture file");
        path
    }

    fn project(&self) -> PathBuf {
        self.write("demo.fpasprj", "[project]\nname = 'demo'\nkind = 'program'\nmain = 'main.fpas'\n[sources]\ninclude = ['*.fpas']\n")
    }
}

impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

fn records(error: &ProjectError) -> Vec<FileDiagnostic> {
    error
        .diagnostics()
        .iter()
        .cloned()
        .map(|diagnostic| {
            FileDiagnostic::new(diagnostic, error.source_path().map(Path::to_path_buf))
        })
        .collect()
}

#[test]
fn graph_failure_preserves_every_field_and_the_graph_source_id() {
    let fixture = Fixture::new("graph");
    let manifest = fixture.project();
    let source = "program Demo; uses Demo.Dep; begin end.\n";
    let main = fixture.write("main.fpas", source);
    let unit_source = "unit Demo.Dep;\nuses Demo.Missing;\nend unit;\n";
    let dependency = fixture.write("dep.fpas", unit_source);
    let loaded = fpas_project::load_project(&manifest).expect("loaded project");
    let (CompilationUnit::Unit(unit), errors) = fpas_parser::parse_compilation_unit(unit_source)
    else {
        panic!("unit fixture");
    };
    assert!(errors.is_empty());
    let graph = fpas_project::build_unit_graph_for_program_from_parsed_sources(
        &main,
        vec![(dependency.clone(), unit)],
        &loaded.link_meta,
    )
    .expect("graph");
    let original = fpas_project::resolve_library_units(&graph).expect_err("missing import");
    let mut service = LanguageService::load(&manifest);
    service
        .documents_mut()
        .open_document(&main, 1, source)
        .expect("editor buffer");
    let analysis = service
        .analyze_document_diagnostics(&main)
        .expect("recoverable analysis");
    let forwarded = analysis.failure().expect("project failure").diagnostics();
    assert_eq!(forwarded, records(&original));
    assert_ne!(
        forwarded[0]
            .diagnostic
            .span
            .expect("import span")
            .source_id(),
        0
    );
    assert_eq!(
        analysis
            .failure_snapshot()
            .expect("failing source snapshot")
            .path(),
        dependency
    );
}

#[test]
fn discovery_preserves_original_parse_records_and_expected_found_details() {
    let fixture = Fixture::new("parse");
    let manifest = fixture.project();
    let source = "program Demo; begin end.\n";
    let main = fixture.write("main.fpas", source);
    fixture.write("dep.fpas", "unit Demo.Dep\nend unit;\n");
    let original = fpas_project::load_project(&manifest).expect_err("missing semicolon");
    let mut service = LanguageService::load(&manifest);
    service
        .documents_mut()
        .open_document(&main, 1, source)
        .expect("editor buffer");
    let analysis = service
        .analyze_document_diagnostics(&main)
        .expect("recoverable discovery");
    let forwarded = analysis.failure().expect("parse failure").diagnostics();
    assert_eq!(forwarded, records(&original));
    assert!(forwarded.iter().any(|record| record.diagnostic.expected.is_some()
        && record.diagnostic.found.is_some()));
}

#[test]
fn standard_library_loading_preserves_the_original_manifest_failure() {
    let fixture = Fixture::new("stdlib");
    let main = fixture.write("main.fpas", "program Demo; begin end.\n");
    fixture.write(
        "lib/stdlib.fpasprj",
        "[project]\nname = 'stdlib'\nkind = 'invalid'\n[sources]\ninclude = ['Std/*.fpas']\n",
    );
    let root = fixture.0.join("lib");
    let original =
        fpas_project::load_standard_library_project(&root).expect_err("bad library manifest");
    let error = LanguageService::load_with_standard_library(&main, &root)
        .err()
        .expect("recoverable library failure");
    assert_eq!(error.diagnostics(), records(&original));
}
