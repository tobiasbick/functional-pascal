//! Source failures survive project loading, dependency traversal and snapshot parsing;
//! manifest, workspace and graph failures carry distinct project codes.

#![allow(
    clippy::expect_used,
    clippy::panic,
    reason = "filesystem fixtures and record shapes identify failures explicitly"
)]

use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

use fpas_diagnostics::{
    DiagnosticCode, DiagnosticStage,
    codes::{
        PROJECT_DEPENDENCY_CYCLE, PROJECT_DISCOVERY_FAILED, PROJECT_DUPLICATE_SOURCE_FILE,
        PROJECT_MANIFEST_READ_FAILED, PROJECT_MANIFEST_SYNTAX_INVALID,
        PROJECT_MANIFEST_VALUE_INVALID, PROJECT_PATH_INVALID, PROJECT_PATTERN_INVALID,
        PROJECT_PROGRAM_SOURCE_SKIPPED, PROJECT_SOURCE_INVALID_UTF8, PROJECT_SOURCE_READ_FAILED,
        PROJECT_UNKNOWN_UNIT,
    },
};
use fpas_project::{
    ProjectLinkMeta, build_unit_graph, build_unit_graph_for_program,
    discover_run_project_in_workspace, load_project, load_standard_library,
    load_standard_library_project, resolve_library_units,
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
    fixture.write("main.fpas", r#"program Demo; begin null; end program;"#);
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
    let unit = fixture.write(
        "broken.fpas",
        r#"unit Broken;
end unit;
"#,
    );
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

/// Asserts one positionless project record with the given code and a help line.
fn assert_validation(error: &fpas_project::ProjectError, code: DiagnosticCode) {
    let [diagnostic] = error.diagnostics() else {
        panic!(
            "expected exactly one record, found {:?}",
            error.diagnostics()
        );
    };
    assert_eq!(diagnostic.code, code);
    assert_eq!(diagnostic.stage(), DiagnosticStage::Project);
    assert_eq!(diagnostic.span, None);
    assert!(diagnostic.help.is_some(), "{diagnostic:?} needs a hint");
    assert!(error.source_path().is_none());
    assert!(!diagnostic.message.contains("help:"));
    let first_line = diagnostic.message.lines().next().unwrap_or_default();
    assert!(
        error
            .to_string()
            .starts_with(&format!("error[{code}]: {first_line}"))
    );
}

#[test]
fn manifest_failures_have_distinct_project_codes() {
    let fixture = Fixture::new();
    let missing = fixture.0.join("missing.fpasprj");
    let syntax = fixture.write("syntax.fpasprj", "[project\n");
    let kind = fixture.write(
        "kind.fpasprj",
        "[project]\nname = 'invalid'\nkind = 'unknown'\n",
    );
    let path = fixture.write(
        "path.fpasprj",
        "[project]\nname = 'p'\nkind = 'library'\n[sources]\ninclude = ['absent.fpas']\n",
    );
    let pattern = fixture.write(
        "pattern.fpasprj",
        "[project]\nname = 'p'\nkind = 'library'\n[sources]\ninclude = ['none/*.fpas']\n",
    );

    for (manifest, code) in [
        (missing, PROJECT_MANIFEST_READ_FAILED),
        (syntax, PROJECT_MANIFEST_SYNTAX_INVALID),
        (kind, PROJECT_MANIFEST_VALUE_INVALID),
        (path, PROJECT_PATH_INVALID),
        (pattern, PROJECT_PATTERN_INVALID),
    ] {
        let error = load_project(&manifest).expect_err("invalid manifest");
        assert_validation(&error, code);
    }
}

#[test]
fn dependency_and_workspace_failures_have_project_codes() {
    let fixture = Fixture::new();
    fixture.write(
        "unit.fpas",
        r#"unit Demo.A;
end unit;

"#,
    );
    let first = fixture.write(
        "first.fpasprj",
        "[project]\nname = 'first'\nkind = 'library'\n[sources]\ninclude = ['unit.fpas']\n[dependencies]\nprojects = ['second.fpasprj']\n",
    );
    fixture.write(
        "second.fpasprj",
        "[project]\nname = 'second'\nkind = 'library'\n[sources]\ninclude = ['unit.fpas']\n[dependencies]\nprojects = ['first.fpasprj']\n",
    );
    let error = load_project(&first).expect_err("dependency cycle");
    assert_validation(&error, PROJECT_DEPENDENCY_CYCLE);

    let workspace = fixture.write(
        "demo.fpasworkspace",
        "[workspace]\nname = 'demo'\nmembers = ['first.fpasprj']\n",
    );
    let error = discover_run_project_in_workspace(&workspace).expect_err("no program member");
    assert_validation(&error, PROJECT_DISCOVERY_FAILED);
}

#[test]
fn unknown_unit_import_is_located_at_its_uses_entry() {
    let fixture = Fixture::new();
    let source = r#"unit Demo.A;
uses Demo.Missing as Missing;
end unit;

"#;
    let unit = fixture.write("a.fpas", source);
    let graph = build_unit_graph(std::slice::from_ref(&unit), &ProjectLinkMeta::default())
        .expect("graph must build");

    let error = resolve_library_units(&graph).expect_err("unknown import");
    let [diagnostic] = error.diagnostics() else {
        panic!("expected one record");
    };
    assert_eq!(diagnostic.code, PROJECT_UNKNOWN_UNIT);
    assert_eq!(error.source_path(), Some(unit.as_path()));
    let span = diagnostic.span.expect("import span");
    assert_eq!((span.line(), span.column()), (2, 6));
    let json = fpas_diagnostics::render_json(
        error.source_path().and_then(Path::to_str),
        Some(source),
        diagnostic,
    )
    .expect("diagnostic JSON");
    assert!(json.contains(r#""code":"F5015""#));
    assert!(json.contains(r#""end":{"line":2,"column":18}"#));
}

#[test]
fn loading_warnings_are_coded_records_with_their_source_path() {
    let fixture = Fixture::new();
    let unit = fixture.write(
        "unit.fpas",
        r#"unit Demo.A;
end unit;

"#,
    );
    fixture.write(
        "tool.fpas",
        r#"program Tool;
begin null;
end program;
"#,
    );
    let manifest = fixture.write(
        "lib.fpasprj",
        "[project]\nname = 'lib'\nkind = 'library'\n[sources]\ninclude = ['unit.fpas', '*.fpas']\n",
    );

    let loaded = load_project(&manifest).expect("warnings do not fail loading");
    let codes = loaded
        .warnings
        .iter()
        .map(|warning| warning.diagnostic.code)
        .collect::<Vec<_>>();
    assert_eq!(
        codes,
        [
            PROJECT_DUPLICATE_SOURCE_FILE,
            PROJECT_PROGRAM_SOURCE_SKIPPED
        ]
    );
    for warning in &loaded.warnings {
        assert!(warning.diagnostic.is_warning());
        assert_eq!(warning.diagnostic.span, None);
        assert!(warning.diagnostic.help.is_some());
        assert!(warning.path.is_some());
    }
    assert_eq!(
        std::fs::canonicalize(loaded.warnings[0].path.as_ref().expect("path")).expect("path"),
        std::fs::canonicalize(&unit).expect("unit")
    );
}
