//! Project failures retain their codes without fabricating document ranges.
//!
//! **Documentation:** `docs/pascal/tools/diagnostics.md`

use super::*;

/// Compares source identity independently of the platform's path separators.
pub(super) fn assert_diagnostic_source(diagnostic: &Value, expected: &std::path::Path) {
    let source = diagnostic["data"]["source"].as_str().expect("source path");
    assert_eq!(std::path::Path::new(source), expected);
}

/// Creates the common program-project fixture for protocol diagnostic tests.
pub(super) fn program_project(temp: &TempDirectory, source: &str) {
    temp.write(
        "demo.fpasprj",
        "[project]\nname = 'demo'\nkind = 'program'\nmain = 'src/main.fpas'\n\n[sources]\ninclude = ['src/**/*.fpas']\n",
    );
    temp.write("src/main.fpas", source);
}

/// Analyzes one opened program through the actual language-server process.
pub(super) fn analyze_project(temp: &TempDirectory, source: &str) -> support::Transcript {
    let root = temp.uri(".");
    let main = temp.uri("src/main.fpas");
    run_script(&[
        TranscriptStep::Message(initialize_with_root(1, Some(&root))),
        TranscriptStep::Message(initialized()),
        TranscriptStep::Message(open(&main, 1, source)),
        TranscriptStep::Wait(ANALYSIS_WAIT),
        TranscriptStep::Message(shutdown(2)),
        TranscriptStep::Message(exit()),
    ])
}

const IMPORT_DEPENDENCY: &str = "program Demo;\nuses Demo.Dep;\nbegin\nend.\n";
const MISSING_IMPORT: &str = "unit Demo.Dep;\nuses Demo.Missing;\nend unit;\n";

/// Finds a coded diagnostic published at its actual source URI.
pub(super) fn located_diagnostic<'a>(
    transcript: &'a support::Transcript,
    uri: &str,
    code: &str,
) -> &'a Value {
    transcript
        .messages
        .iter()
        .filter(|message| {
            message["method"] == "textDocument/publishDiagnostics"
                && message["params"]["uri"] == uri
        })
        .flat_map(|message| {
            message["params"]["diagnostics"]
                .as_array()
                .into_iter()
                .flatten()
        })
        .find(|diagnostic| diagnostic["code"] == code)
        .unwrap_or_else(|| panic!("missing {code} for {uri}: {:?}", transcript.messages))
}

fn assert_unlocated(transcript: &support::Transcript, code: &str, source: Option<&str>) {
    assert_success(transcript);
    let published = notifications(&transcript.messages, "textDocument/publishDiagnostics");
    assert!(
        published
            .iter()
            .all(|message| message["params"]["diagnostics"] == json!([])),
        "{published:?}"
    );
    let logged = notifications(&transcript.messages, "window/logMessage");
    let record = logged
        .iter()
        .find(|message| {
            message["params"]["message"]
                .as_str()
                .is_some_and(|text| text.contains(&format!("error[{code}]")))
        })
        .unwrap_or_else(|| panic!("missing {code}: {logged:?}"));
    assert_eq!(record["params"]["type"], json!(1));
    if let Some(source) = source {
        assert!(
            record["params"]["message"]
                .as_str()
                .is_some_and(|text| text.starts_with(source)),
            "{record:?}"
        );
    }
}

#[test]
fn missing_unit_in_a_dependency_publishes_its_own_uri_and_import_range() {
    let temp = TempDirectory::new("missing-dependency-unit");
    program_project(&temp, IMPORT_DEPENDENCY);
    let dependency = temp.write("src/dep.fpas", MISSING_IMPORT);
    let transcript = analyze_project(&temp, IMPORT_DEPENDENCY);
    assert_success(&transcript);
    let diagnostic = located_diagnostic(&transcript, &temp.uri("src/dep.fpas"), "FP4115");
    assert_eq!(
        diagnostic["range"],
        json!({"start":{"line":1,"character":5},"end":{"line":1,"character":17}})
    );
    assert_eq!(diagnostic["severity"], json!(1));
    assert_eq!(diagnostic["data"]["source_id"], json!(1));
    assert_diagnostic_source(diagnostic, &dependency);
    assert!(diagnostic["message"].as_str().is_some_and(|message| message.starts_with("Unknown unit") && message.contains("Help:")), "{diagnostic:?}");
    assert!(diagnostic["data"]["hint"].is_string());
    assert_eq!(diagnostic["data"]["expected"], Value::Null);
    assert_eq!(diagnostic["data"]["found"], Value::Null);
}

#[test]
fn dependency_parser_records_preserve_expected_found_and_the_source_range() {
    let temp = TempDirectory::new("dependency-parser-records");
    let source = "program Demo; begin end.\n";
    program_project(&temp, source);
    let dependency = temp.write("src/dep.fpas", "unit Demo.Dep\nend unit;\n");
    let transcript = analyze_project(&temp, source);
    assert_success(&transcript);
    let diagnostic = located_diagnostic(&transcript, &temp.uri("src/dep.fpas"), "FP2001");
    assert_diagnostic_source(diagnostic, &dependency);
    assert_eq!(diagnostic["data"]["expected"], json!(";"));
    assert_eq!(diagnostic["data"]["found"], json!("end"));
    assert_eq!(
        diagnostic["range"],
        json!({"start":{"line":1,"character":0},"end":{"line":1,"character":3}})
    );
}

#[test]
fn nonexported_dependency_import_preserves_code_source_and_range() {
    let temp = TempDirectory::new("nonexported-unit");
    program_project(&temp, IMPORT_DEPENDENCY);
    temp.write("demo.fpasprj", "[project]\nname = 'demo'\nkind = 'program'\nmain = 'src/main.fpas'\n[sources]\ninclude = ['src/**/*.fpas']\n[dependencies]\nprojects = ['library/library.fpasprj']\n");
    temp.write("library/library.fpasprj", "[project]\nname = 'library'\nkind = 'library'\n[sources]\ninclude = ['src/*.fpas']\n[exports]\nunits = ['Library.Api']\n");
    temp.write("library/src/api.fpas", "unit Library.Api; end unit;\n");
    temp.write(
        "library/src/hidden.fpas",
        "unit Library.Hidden; end unit;\n",
    );
    let dependency = temp.write(
        "src/dep.fpas",
        "unit Demo.Dep;\nuses Library.Hidden;\nend unit;\n",
    );
    let transcript = analyze_project(&temp, IMPORT_DEPENDENCY);
    assert_success(&transcript);
    let diagnostic = located_diagnostic(&transcript, &temp.uri("src/dep.fpas"), "FP4116");
    assert_eq!(
        diagnostic["range"],
        json!({"start":{"line":1,"character":5},"end":{"line":1,"character":19}})
    );
    assert_diagnostic_source(diagnostic, &dependency);
    assert!(
        diagnostic["data"]["hint"]
            .as_str()
            .is_some_and(|hint| hint.contains("[exports].units")),
        "{diagnostic:?}"
    );
}

#[test]
fn unit_cycle_retains_its_original_code_without_inventing_a_location() {
    let temp = TempDirectory::new("cyclic-units");
    program_project(&temp, IMPORT_DEPENDENCY);
    temp.write(
        "src/dep.fpas",
        "unit Demo.Dep; uses Demo.Other; end unit;\n",
    );
    temp.write(
        "src/other.fpas",
        "unit Demo.Other; uses Demo.Dep; end unit;\n",
    );
    let transcript = analyze_project(&temp, IMPORT_DEPENDENCY);
    assert_unlocated(&transcript, "FP4117", None);
}

#[test]
fn dependency_manifest_failure_retains_the_manifest_path_and_code() {
    let temp = TempDirectory::new("invalid-dependency-manifest");
    let source = "program Demo; begin end.\n";
    program_project(&temp, source);
    temp.write("demo.fpasprj", "[project]\nname = 'demo'\nkind = 'program'\nmain = 'src/main.fpas'\n[sources]\ninclude = ['src/**/*.fpas']\n[dependencies]\nprojects = ['library/library.fpasprj']\n");
    let manifest = temp.write(
        "library/library.fpasprj",
        "[project]\nname = 'library'\nkind = 'invalid'\n[sources]\ninclude = ['src/*.fpas']\n",
    );
    temp.write("library/src/api.fpas", "unit Library.Api; end unit;\n");
    let transcript = analyze_project(&temp, source);
    assert_unlocated(
        &transcript,
        "FP4105",
        Some(manifest.to_string_lossy().as_ref()),
    );
}

#[test]
fn correcting_a_dependency_clears_its_previous_project_markers() {
    let temp = TempDirectory::new("correct-dependency-error");
    program_project(&temp, IMPORT_DEPENDENCY);
    let dependency = temp.write("src/dep.fpas", MISSING_IMPORT);
    let root = temp.uri(".");
    let main = temp.uri("src/main.fpas");
    let transcript = run_script(&[
        TranscriptStep::Message(initialize_with_root(1, Some(&root))),
        TranscriptStep::Message(initialized()),
        TranscriptStep::Message(open(&main, 1, IMPORT_DEPENDENCY)),
        TranscriptStep::Wait(ANALYSIS_WAIT),
        TranscriptStep::Action(Box::new(move || {
            std::fs::write(&dependency, "unit Demo.Dep; end unit;\n").expect("fix dependency")
        })),
        TranscriptStep::Message(change(&main, 2, IMPORT_DEPENDENCY)),
        TranscriptStep::Wait(ANALYSIS_WAIT),
        TranscriptStep::Message(shutdown(2)),
        TranscriptStep::Message(exit()),
    ]);
    assert_success(&transcript);
    located_diagnostic(&transcript, &temp.uri("src/dep.fpas"), "FP4115");
    let dependency_uri = temp.uri("src/dep.fpas");
    let published = notifications(&transcript.messages, "textDocument/publishDiagnostics");
    let last = published
        .iter()
        .rev()
        .find(|message| message["params"]["uri"] == dependency_uri)
        .expect("dependency publication");
    assert_eq!(last["params"]["diagnostics"], json!([]));
    assert_eq!(
        publication(&published, 2)["params"]["diagnostics"],
        json!([])
    );
}

#[test]
fn closing_the_origin_clears_its_related_dependency_markers() {
    let temp = TempDirectory::new("close-dependency-error");
    program_project(&temp, IMPORT_DEPENDENCY);
    temp.write("src/dep.fpas", MISSING_IMPORT);
    let root = temp.uri(".");
    let main = temp.uri("src/main.fpas");
    let transcript = run_script(&[
        TranscriptStep::Message(initialize_with_root(1, Some(&root))),
        TranscriptStep::Message(initialized()),
        TranscriptStep::Message(open(&main, 1, IMPORT_DEPENDENCY)),
        TranscriptStep::Wait(ANALYSIS_WAIT),
        TranscriptStep::Message(close(&main)),
        TranscriptStep::Wait(ANALYSIS_WAIT),
        TranscriptStep::Message(shutdown(2)),
        TranscriptStep::Message(exit()),
    ]);
    assert_success(&transcript);
    located_diagnostic(&transcript, &temp.uri("src/dep.fpas"), "FP4115");
    let dependency_uri = temp.uri("src/dep.fpas");
    let published = notifications(&transcript.messages, "textDocument/publishDiagnostics");
    let last = published
        .iter()
        .rev()
        .find(|message| message["params"]["uri"] == dependency_uri)
        .expect("dependency clear");
    assert_eq!(last["params"]["diagnostics"], json!([]));
}

#[test]
fn closing_one_origin_preserves_another_origins_dependency_diagnostics() {
    let temp = TempDirectory::new("shared-dependency-error");
    program_project(&temp, IMPORT_DEPENDENCY);
    temp.write("src/dep.fpas", MISSING_IMPORT);
    let root = temp.uri(".");
    let main = temp.uri("src/main.fpas");
    let dependency_uri = temp.uri("src/dep.fpas");
    let transcript = run_script(&[
        TranscriptStep::Message(initialize_with_root(1, Some(&root))),
        TranscriptStep::Message(initialized()),
        TranscriptStep::Message(open(&main, 1, IMPORT_DEPENDENCY)),
        TranscriptStep::Wait(ANALYSIS_WAIT),
        TranscriptStep::Message(open(&dependency_uri, 1, MISSING_IMPORT)),
        TranscriptStep::Wait(ANALYSIS_WAIT),
        TranscriptStep::Message(close(&main)),
        TranscriptStep::Wait(ANALYSIS_WAIT),
        TranscriptStep::Message(shutdown(2)),
        TranscriptStep::Message(exit()),
    ]);
    assert_success(&transcript);
    let published = notifications(&transcript.messages, "textDocument/publishDiagnostics");
    let last = published
        .iter()
        .rev()
        .find(|message| message["params"]["uri"] == dependency_uri)
        .expect("remaining dependency diagnostics");
    let diagnostics = last["params"]["diagnostics"]
        .as_array()
        .expect("diagnostics");
    assert_eq!(diagnostics.len(), 1, "{last:?}");
    assert_eq!(diagnostics[0]["code"], json!("FP4115"));
}

#[test]
fn reopened_dependency_version_ignores_project_records_from_an_older_snapshot() {
    let temp = TempDirectory::new("reopened-dependency-error");
    program_project(&temp, IMPORT_DEPENDENCY);
    temp.write("src/dep.fpas", MISSING_IMPORT);
    let root = temp.uri(".");
    let main = temp.uri("src/main.fpas");
    let dependency_uri = temp.uri("src/dep.fpas");
    let transcript = run_script(&[
        TranscriptStep::Message(initialize_with_root(1, Some(&root))),
        TranscriptStep::Message(initialized()),
        TranscriptStep::Message(open(&dependency_uri, 1, MISSING_IMPORT)),
        TranscriptStep::Wait(ANALYSIS_WAIT),
        TranscriptStep::Message(open(&main, 1, IMPORT_DEPENDENCY)),
        TranscriptStep::Wait(ANALYSIS_WAIT),
        TranscriptStep::Message(close(&dependency_uri)),
        TranscriptStep::Message(open(&dependency_uri, 1, "unit Demo.Dep; end unit;\n")),
        TranscriptStep::Wait(ANALYSIS_WAIT),
        TranscriptStep::Message(shutdown(2)),
        TranscriptStep::Message(exit()),
    ]);
    assert_success(&transcript);
    located_diagnostic(&transcript, &dependency_uri, "FP4115");
    let published = notifications(&transcript.messages, "textDocument/publishDiagnostics");
    let last = published
        .iter()
        .rev()
        .find(|message| message["params"]["uri"] == dependency_uri)
        .expect("reopened dependency publication");
    assert_eq!(last["params"]["version"], json!(1));
    assert_eq!(last["params"]["diagnostics"], json!([]));
}

#[test]
fn unsaved_dependency_correction_does_not_republish_disk_parser_errors() {
    let temp = TempDirectory::new("unsaved-dependency-correction");
    let main_source =
        "program Demo;\nuses Demo.Dep;\nbegin\n  const Value: integer := 'wrong';\nend.\n";
    program_project(&temp, main_source);
    let broken = "unit Demo.Dep\nend unit;\n";
    temp.write("src/dep.fpas", broken);
    let root = temp.uri(".");
    let main = temp.uri("src/main.fpas");
    let dependency = temp.uri("src/dep.fpas");
    let transcript = run_script(&[
        TranscriptStep::Message(initialize_with_root(1, Some(&root))),
        TranscriptStep::Message(initialized()),
        TranscriptStep::Message(open(&dependency, 1, broken)),
        TranscriptStep::Wait(ANALYSIS_WAIT),
        TranscriptStep::Message(change(&dependency, 2, "unit Demo.Dep;\nend unit;\n")),
        TranscriptStep::Wait(ANALYSIS_WAIT),
        TranscriptStep::Message(open(&main, 1, main_source)),
        TranscriptStep::Wait(ANALYSIS_WAIT),
        TranscriptStep::Message(shutdown(2)),
        TranscriptStep::Message(exit()),
    ]);
    assert_success(&transcript);
    located_diagnostic(&transcript, &dependency, "FP2001");
    let published = notifications(&transcript.messages, "textDocument/publishDiagnostics");
    for uri in [&dependency, &main] {
        let last = published
            .iter()
            .rev()
            .find(|message| message["params"]["uri"] == *uri)
            .expect("current diagnostic publication");
        if uri == &dependency {
            assert_eq!(last["params"]["diagnostics"], json!([]), "{last:?}");
        } else {
            let diagnostics = last["params"]["diagnostics"]
                .as_array()
                .expect("diagnostics");
            assert!(
                diagnostics.iter().any(|diagnostic| diagnostic["code"]
                    .as_str()
                    .is_some_and(|code| code.starts_with("FP3"))),
                "{last:?}"
            );
        }
    }
}
