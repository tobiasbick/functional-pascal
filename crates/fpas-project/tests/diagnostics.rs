//! Source failures survive project loading, dependency traversal and snapshot parsing.

#![allow(
    clippy::expect_used,
    reason = "filesystem fixtures identify setup failures explicitly"
)]

use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

use fpas_diagnostics::{
    DiagnosticStage,
    codes::{PROJECT_SOURCE_INVALID_UTF8, PROJECT_SOURCE_READ_FAILED},
};
use fpas_project::{
    ProjectLinkMeta, build_unit_graph, build_unit_graph_for_program, load_project,
    load_standard_library, load_standard_library_project,
};

const INVALID_PROGRAM: &str = "program Broken\nbegin\n  var X := ;\n  §\nend.";
const INVALID_UNIT: &str = "unit Broken\npublic const A: integer := ;\n§";

struct Fixture(PathBuf);

impl Fixture {
    fn new() -> Self {
        static NEXT: AtomicU64 = AtomicU64::new(0);
        let id = NEXT.fetch_add(1, Ordering::Relaxed);
        let path = std::env::temp_dir().join(format!(
            "fpas-project-diagnostics-{}-{id}",
            std::process::id()
        ));
        std::fs::create_dir_all(&path).expect("fixture directory");
        Self(path)
    }

    fn write(&self, name: &str, contents: impl AsRef<[u8]>) -> PathBuf {
        let path = self.0.join(name);
        std::fs::write(&path, contents).expect("fixture source");
        path
    }
}

impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

fn assert_source(error: &fpas_project::ProjectError, path: &Path, source: &str) {
    assert_eq!(
        std::fs::canonicalize(error.source_path().expect("source path"))
            .expect("diagnostic source"),
        std::fs::canonicalize(path).expect("expected source")
    );
    let (_, expected) = fpas_parser::parse_compilation_unit(source);
    assert!(expected.len() > 1);
    assert_eq!(
        error.diagnostics(),
        expected
            .iter()
            .map(|error| error.as_diagnostic().clone())
            .collect::<Vec<_>>()
    );
    assert!(
        error
            .diagnostics()
            .iter()
            .any(|error| error.stage() == DiagnosticStage::Lex)
    );
    assert!(
        error
            .diagnostics()
            .iter()
            .any(|error| error.expected.is_some())
    );
    let text = error.to_string();
    for diagnostic in error.diagnostics() {
        assert!(text.contains(&diagnostic.code.to_string()));
    }
}

#[test]
fn project_main_preserves_lexer_and_parser_records_in_order() {
    let fixture = Fixture::new();
    let main = fixture.write("main.fpas", INVALID_PROGRAM);
    let manifest = fixture.write("app.fpasprj", "[project]\nname = 'app'\nkind = 'program'\nmain = 'main.fpas'\n[sources]\ninclude = ['main.fpas']\n");
    let error = load_project(&manifest).expect_err("invalid main source");
    assert_source(&error, &main, INVALID_PROGRAM);
}

#[test]
fn dependency_errors_keep_the_dependency_file_instead_of_the_root_manifest() {
    let fixture = Fixture::new();
    let unit = fixture.write("broken.fpas", INVALID_UNIT);
    fixture.write(
        "library.fpasprj",
        "[project]\nname = 'library'\nkind = 'library'\n[sources]\ninclude = ['broken.fpas']\n",
    );
    fixture.write("main.fpas", "program Demo; begin end.");
    let root = fixture.write("app.fpasprj", "[project]\nname = 'app'\nkind = 'program'\nmain = 'main.fpas'\n[sources]\ninclude = ['main.fpas']\n[dependencies]\nprojects = ['library.fpasprj']\n");
    let error = load_project(&root).expect_err("invalid dependency source");
    assert_source(&error, &unit, INVALID_UNIT);
}

#[test]
fn graph_read_and_utf8_errors_have_codes_and_paths_without_invented_positions() {
    let fixture = Fixture::new();
    let missing = fixture.0.join("missing.fpas");
    let invalid = fixture.write("invalid.fpas", [0xff, 0xfe]);
    for (path, code) in [
        (&missing, PROJECT_SOURCE_READ_FAILED),
        (&invalid, PROJECT_SOURCE_INVALID_UTF8),
    ] {
        let error = build_unit_graph(std::slice::from_ref(path), &ProjectLinkMeta::default())
            .expect_err("unreadable source");
        assert_eq!(error.source_path(), Some(path.as_path()));
        assert_eq!(error.diagnostics().len(), 1);
        let diagnostic = &error.diagnostics()[0];
        assert_eq!(diagnostic.code, code);
        assert_eq!(diagnostic.span, None);
        assert!(
            fpas_diagnostics::render_json(Some(&path.to_string_lossy()), None, diagnostic)
                .expect("JSON")
                .contains("\"location\":null")
        );
    }
}

#[test]
fn graph_parser_errors_keep_all_records() {
    let fixture = Fixture::new();
    let unit = fixture.write("broken.fpas", INVALID_UNIT);
    let error = build_unit_graph(std::slice::from_ref(&unit), &ProjectLinkMeta::default())
        .expect_err("invalid graph source");
    assert_source(&error, &unit, INVALID_UNIT);
}

#[test]
fn snapshot_errors_keep_the_existing_graph_source_id() {
    let fixture = Fixture::new();
    let unit = fixture.write("broken.fpas", "unit Broken;");
    let graph = build_unit_graph_for_program(
        &fixture.0.join("main.fpas"),
        std::slice::from_ref(&unit),
        &ProjectLinkMeta::default(),
    )
    .expect("valid graph");
    let node = graph.get("broken").expect("unit node");
    assert_ne!(node.source_id(), 0);
    let error = node
        .parse_source_snapshot(INVALID_UNIT.as_bytes())
        .expect_err("invalid snapshot");
    assert_eq!(error.source_path(), Some(unit.as_path()));
    assert!(error.diagnostics().len() > 1);
    for diagnostic in error.diagnostics() {
        assert_eq!(
            diagnostic.span.expect("source span").source_id(),
            node.source_id()
        );
    }
}

#[test]
fn trusted_standard_library_loaders_preserve_source_diagnostics() {
    let fixture = Fixture::new();
    fixture.write(
        "stdlib.fpasprj",
        "[project]\nname = 'stdlib'\nkind = 'library'\n[sources]\ninclude = ['broken.fpas']\n",
    );
    let source = INVALID_UNIT.replace("unit Broken", "unit Std.Broken");
    let unit = fixture.write("broken.fpas", &source);
    let errors = [
        load_standard_library(&fixture.0).expect_err("invalid standard unit"),
        load_standard_library_project(&fixture.0).expect_err("invalid editable standard unit"),
    ];
    for error in errors {
        assert_source(&error, &unit, &source);
    }
}

#[test]
fn manifest_validation_errors_remain_distinct_from_source_diagnostics() {
    let fixture = Fixture::new();
    let manifest = fixture.write(
        "invalid.fpasprj",
        "[project]\nname = 'invalid'\nkind = 'unknown'\n",
    );
    let error = load_project(&manifest).expect_err("invalid manifest");
    assert!(error.diagnostics().is_empty());
    assert!(error.source_path().is_none());
    assert!(error.to_string().contains("unknown"));
}
