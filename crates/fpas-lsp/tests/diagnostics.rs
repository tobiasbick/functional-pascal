//! Protocol-level integration tests for LSP diagnostics and document updates.

#![allow(
    clippy::expect_used,
    clippy::panic,
    reason = "protocol fixtures use explicit assertions for hard-coded LSP transcripts"
)]

mod support;

use std::time::Duration;

use serde_json::{Value, json};

use support::{
    TempDirectory, TranscriptStep, exit, initialize, initialize_with_root, initialized,
    notifications, run_script, shutdown,
};

const ANALYSIS_WAIT: Duration = Duration::from_millis(260);

#[test]
fn invalid_comment_form_publishes_one_actionable_lexer_diagnostic() {
    let uri = "file:///comments/invalid.fpas";
    let transcript = run_script(&[
        TranscriptStep::Message(initialize(1)),
        TranscriptStep::Message(initialized()),
        TranscriptStep::Message(open(
            uri,
            1,
            "program Invalid;\n{ not a comment }\nbegin\nend.\n",
        )),
        TranscriptStep::Wait(ANALYSIS_WAIT),
        TranscriptStep::Message(shutdown(2)),
        TranscriptStep::Message(exit()),
    ]);

    assert_success(&transcript);
    let published = notifications(&transcript.messages, "textDocument/publishDiagnostics");
    let diagnostics = publication(&published, 1)["params"]["diagnostics"]
        .as_array()
        .expect("diagnostic array");
    assert_eq!(diagnostics.len(), 1, "{diagnostics:?}");
    assert_eq!(diagnostics[0]["code"], json!("F0013"));
    assert!(
        diagnostics[0]["message"]
            .as_str()
            .is_some_and(|message| message.contains("Use `// comment`")),
        "{diagnostics:?}"
    );
}

#[test]
fn parser_and_semantic_errors_publish_and_a_fixed_version_clears_them() {
    let uri = "file:///phase5/diagnostics.fpas";
    let transcript = run_script(&[
        TranscriptStep::Message(initialize(1)),
        TranscriptStep::Message(initialized()),
        TranscriptStep::Message(open(uri, 1, "program Broken;\nbegin\n  if then\nend.\n")),
        TranscriptStep::Wait(ANALYSIS_WAIT),
        TranscriptStep::Message(change(
            uri,
            2,
            "program Semantic;\nbegin\n  var Value: integer := 'wrong'\nend.\n",
        )),
        TranscriptStep::Wait(ANALYSIS_WAIT),
        TranscriptStep::Message(change(
            uri,
            3,
            "program Fixed;\nbegin\n  var Value: integer := 1\nend.\n",
        )),
        TranscriptStep::Wait(ANALYSIS_WAIT),
        TranscriptStep::Message(shutdown(2)),
        TranscriptStep::Message(exit()),
    ]);

    assert_success(&transcript);
    let published = notifications(&transcript.messages, "textDocument/publishDiagnostics");
    let version_1 = publication(&published, 1);
    assert!(
        version_1["params"]["diagnostics"]
            .as_array()
            .expect("parser diagnostics")
            .iter()
            .any(|diagnostic| diagnostic["code"]
                .as_str()
                .is_some_and(|code| code.starts_with("F1"))),
        "{version_1:?}"
    );
    assert!(
        version_1["params"]["diagnostics"]
            .as_array()
            .expect("parser diagnostics")
            .iter()
            .any(|diagnostic| diagnostic["message"]
                .as_str()
                .is_some_and(|message| message.contains("Help:"))),
        "{version_1:?}"
    );

    let version_2 = publication(&published, 2);
    assert!(
        version_2["params"]["diagnostics"]
            .as_array()
            .expect("semantic diagnostics")
            .iter()
            .any(|diagnostic| {
                diagnostic["code"]
                    .as_str()
                    .is_some_and(|code| code.starts_with("F2"))
                    && diagnostic["severity"] == json!(1)
            }),
        "{version_2:?}"
    );

    assert_eq!(
        publication(&published, 3)["params"]["diagnostics"],
        json!([])
    );
}

#[test]
fn rapid_changes_publish_only_the_latest_document_version() {
    let uri = "file:///phase5/rapid.fpas";
    let transcript = run_script(&[
        TranscriptStep::Message(initialize(1)),
        TranscriptStep::Message(initialized()),
        TranscriptStep::Message(open(uri, 1, "program First;\nbegin\n  if then\nend.\n")),
        TranscriptStep::Message(change(
            uri,
            2,
            "program Second;\nbegin\n  var Value: integer := 'wrong'\nend.\n",
        )),
        TranscriptStep::Message(change(
            uri,
            3,
            "program Latest;\nbegin\n  var Value: integer := 1\nend.\n",
        )),
        TranscriptStep::Wait(ANALYSIS_WAIT),
        TranscriptStep::Message(shutdown(2)),
        TranscriptStep::Message(exit()),
    ]);

    assert_success(&transcript);
    let published = notifications(&transcript.messages, "textDocument/publishDiagnostics");
    assert_eq!(published.len(), 1, "{published:?}");
    assert_eq!(published[0]["params"]["version"], json!(3));
    assert_eq!(published[0]["params"]["diagnostics"], json!([]));
}

#[test]
fn multiple_diagnostics_keep_distinct_ranges() {
    let uri = "file:///phase5/multiple.fpas";
    let transcript = run_script(&[
        TranscriptStep::Message(initialize(1)),
        TranscriptStep::Message(initialized()),
        TranscriptStep::Message(open(
            uri,
            1,
            "program Multi\nbegin\n  if then\n  var :=\nend.\n",
        )),
        TranscriptStep::Wait(ANALYSIS_WAIT),
        TranscriptStep::Message(shutdown(2)),
        TranscriptStep::Message(exit()),
    ]);

    assert_success(&transcript);
    let published = notifications(&transcript.messages, "textDocument/publishDiagnostics");
    let diagnostics = publication(&published, 1)["params"]["diagnostics"]
        .as_array()
        .expect("diagnostic array");
    assert!(diagnostics.len() >= 2, "{diagnostics:?}");
    assert!(
        diagnostics
            .iter()
            .all(|diagnostic| diagnostic["range"]["start"] != Value::Null),
        "{diagnostics:?}"
    );
    assert!(
        diagnostics
            .windows(2)
            .any(|pair| pair[0]["range"] != pair[1]["range"]),
        "{diagnostics:?}"
    );
}

#[test]
fn project_dependency_diagnostic_uses_the_dependency_uri_and_range() {
    let temp = TempDirectory::new("dependency-diagnostic");
    temp.write(
        "demo.fpasprj",
        r#"[project]
name = "demo"
kind = "program"
main = "src/main.fpas"

[sources]
include = ["src/**/*.fpas"]
"#,
    );
    temp.write(
        "src/main.fpas",
        "program App;\n\nuses Demo.Math;\n\nbegin\n  var Value: integer := Answer()\nend.\n",
    );
    let unit_source =
        "unit Demo.Math;\n\npublic function Answer(): integer;\nbegin\n  return 'wrong'\nend;\n";
    temp.write("src/math.fpas", unit_source);
    let root_uri = temp.uri(".");
    let unit_uri = temp.uri("src/math.fpas");
    let transcript = run_script(&[
        TranscriptStep::Message(initialize_with_root(1, Some(&root_uri))),
        TranscriptStep::Message(initialized()),
        TranscriptStep::Message(open(&unit_uri, 1, unit_source)),
        TranscriptStep::Wait(ANALYSIS_WAIT),
        TranscriptStep::Message(shutdown(2)),
        TranscriptStep::Message(exit()),
    ]);

    assert_success(&transcript);
    let published = notifications(&transcript.messages, "textDocument/publishDiagnostics");
    let publication = publication(&published, 1);
    assert_eq!(publication["params"]["uri"], json!(unit_uri));
    let diagnostics = publication["params"]["diagnostics"]
        .as_array()
        .expect("dependency diagnostics");
    assert!(
        diagnostics.iter().any(|diagnostic| {
            diagnostic["code"]
                .as_str()
                .is_some_and(|code| code.starts_with("F2"))
                && diagnostic["range"]["start"]["line"] == json!(4)
        }),
        "{diagnostics:?}"
    );
}

#[test]
fn missing_sibling_publishes_current_syntax_and_project_io_diagnostics() {
    let temp = TempDirectory::new("missing-sibling-diagnostics");
    temp.write(
        "demo.fpasprj",
        r#"[project]
name = "demo"
kind = "program"
main = "src/main.fpas"

[sources]
include = ["src/**/*.fpas"]
"#,
    );
    let valid =
        "program App;\n\nuses Demo.Math;\n\nbegin\n  var Value: integer := Answer()\nend.\n";
    let unit_source =
        "unit Demo.Math;\n\npublic function Answer(): integer;\nbegin\n  return 42\nend;\n";
    let main = temp.write("src/main.fpas", valid);
    let unit = temp.write("src/math.fpas", unit_source);
    let root_uri = temp.uri(".");
    let main_uri = temp.uri("src/main.fpas");
    let remove_unit = unit.clone();
    let restore_unit = unit.clone();
    let transcript = run_script(&[
        TranscriptStep::Message(initialize_with_root(1, Some(&root_uri))),
        TranscriptStep::Message(initialized()),
        TranscriptStep::Message(open(&main_uri, 1, valid)),
        TranscriptStep::Wait(ANALYSIS_WAIT),
        TranscriptStep::Action(Box::new(move || {
            std::fs::remove_file(&remove_unit).expect("remove sibling")
        })),
        TranscriptStep::Message(change(
            &main_uri,
            2,
            "program Broken;\nbegin\n  if then\nend.\n",
        )),
        TranscriptStep::Wait(ANALYSIS_WAIT),
        TranscriptStep::Action(Box::new(move || {
            std::fs::write(&restore_unit, unit_source).expect("restore sibling")
        })),
        TranscriptStep::Message(change(&main_uri, 3, valid)),
        TranscriptStep::Wait(ANALYSIS_WAIT),
        TranscriptStep::Message(shutdown(2)),
        TranscriptStep::Message(exit()),
    ]);

    assert_success(&transcript);
    let published = notifications(&transcript.messages, "textDocument/publishDiagnostics");
    let version_2 = publication(&published, 2)["params"]["diagnostics"]
        .as_array()
        .expect("version two diagnostics");
    assert!(
        version_2
            .iter()
            .any(|diagnostic| diagnostic["code"] == "FPAS_PROJECT_IO"),
        "{version_2:?}"
    );
    assert!(
        version_2.iter().any(|diagnostic| diagnostic["code"]
            .as_str()
            .is_some_and(|code| code.starts_with("F1"))),
        "{version_2:?}"
    );
    assert_eq!(
        publication(&published, 3)["params"]["diagnostics"],
        json!([])
    );
    assert!(main.exists());
}

#[test]
fn repository_root_discovers_a_nested_standard_library_before_diagnostics() {
    let temp = TempDirectory::new("nested-standard-library");
    temp.write(
        "repository/lib/stdlib.fpasprj",
        r#"[project]
name = "test-stdlib"
kind = "library"

[sources]
include = ["Std/**/*.fpas"]
"#,
    );
    temp.write(
        "repository/lib/Std/Point.fpas",
        r#"unit Std.Point;

public type Point = record
  public X: integer;
end;
"#,
    );
    let facade_source = r#"unit Std.Facade;

uses Std.Point;

public type FacadePoint = Std.Point.Point;
"#;
    temp.write("repository/lib/Std/Facade.fpas", facade_source);
    let root_uri = temp.uri("repository");
    let facade_uri = temp.uri("repository/lib/Std/Facade.fpas");
    let transcript = run_script(&[
        TranscriptStep::Message(initialize_with_root(1, Some(&root_uri))),
        TranscriptStep::Message(initialized()),
        TranscriptStep::Message(open(&facade_uri, 1, facade_source)),
        TranscriptStep::Wait(ANALYSIS_WAIT),
        TranscriptStep::Message(shutdown(2)),
        TranscriptStep::Message(exit()),
    ]);

    assert_success(&transcript);
    let published = notifications(&transcript.messages, "textDocument/publishDiagnostics");
    assert_eq!(
        publication(&published, 1)["params"]["diagnostics"],
        json!([])
    );
}

#[test]
fn initialized_standard_library_resolves_std_units_for_an_external_project() {
    let temp = TempDirectory::new("configured-standard-library");
    temp.write(
        "bundle/stdlib.fpasprj",
        r#"[project]
name = "test-stdlib"
kind = "library"

[exports]
units = ["Std.Tui"]

[sources]
include = ["Std/**/*.fpas"]
"#,
    );
    temp.write(
        "bundle/Std/Tui.fpas",
        "unit Std.Tui;\n\npublic type TuiPalette = integer;\n",
    );
    temp.write(
        "external/external.fpasprj",
        r#"[project]
name = "external"
kind = "program"
main = "main.fpas"

[sources]
include = ["main.fpas"]
"#,
    );
    let source =
        "program External;\n\nuses Std.Tui;\n\nbegin\n  var Palette: TuiPalette := 1\nend.\n";
    temp.write("external/main.fpas", source);
    let root_uri = temp.uri("external");
    let standard_library_uri = temp.uri("bundle");
    let source_uri = temp.uri("external/main.fpas");
    let transcript = run_script(&[
        TranscriptStep::Message(json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "initialize",
            "params": {
                "processId": null,
                "rootUri": root_uri,
                "capabilities": {},
                "initializationOptions": {
                    "standardLibraryUri": standard_library_uri
                }
            }
        })),
        TranscriptStep::Message(initialized()),
        TranscriptStep::Message(open(&source_uri, 1, source)),
        TranscriptStep::Wait(ANALYSIS_WAIT),
        TranscriptStep::Message(shutdown(2)),
        TranscriptStep::Message(exit()),
    ]);

    assert_success(&transcript);
    let published = notifications(&transcript.messages, "textDocument/publishDiagnostics");
    assert_eq!(
        publication(&published, 1)["params"]["diagnostics"],
        json!([])
    );
}

#[test]
fn close_during_debounce_cancels_analysis_and_clears_diagnostics() {
    let uri = "file:///phase5/closed.fpas";
    let transcript = run_script(&[
        TranscriptStep::Message(initialize(1)),
        TranscriptStep::Message(initialized()),
        TranscriptStep::Message(open(uri, 1, "program Closed;\nbegin\n  if then\nend.\n")),
        TranscriptStep::Message(close(uri)),
        TranscriptStep::Wait(ANALYSIS_WAIT),
        TranscriptStep::Message(shutdown(2)),
        TranscriptStep::Message(exit()),
    ]);

    assert_success(&transcript);
    let published = notifications(&transcript.messages, "textDocument/publishDiagnostics");
    assert_eq!(published.len(), 1, "{published:?}");
    assert_eq!(published[0]["params"]["uri"], json!(uri));
    assert_eq!(published[0]["params"]["diagnostics"], json!([]));
    assert_eq!(published[0]["params"].get("version"), None);
}

fn open(uri: &str, version: i32, text: &str) -> Value {
    json!({
        "jsonrpc": "2.0",
        "method": "textDocument/didOpen",
        "params": {
            "textDocument": {
                "uri": uri,
                "languageId": "fpas",
                "version": version,
                "text": text
            }
        }
    })
}

fn change(uri: &str, version: i32, text: &str) -> Value {
    json!({
        "jsonrpc": "2.0",
        "method": "textDocument/didChange",
        "params": {
            "textDocument": {"uri": uri, "version": version},
            "contentChanges": [{"text": text}]
        }
    })
}

fn close(uri: &str) -> Value {
    json!({
        "jsonrpc": "2.0",
        "method": "textDocument/didClose",
        "params": {"textDocument": {"uri": uri}}
    })
}

fn publication<'a>(published: &[&'a Value], version: i32) -> &'a Value {
    published
        .iter()
        .copied()
        .find(|message| message["params"]["version"] == json!(version))
        .unwrap_or_else(|| {
            panic!("missing diagnostic publication for version {version}: {published:?}")
        })
}

fn assert_success(transcript: &support::Transcript) {
    assert!(
        transcript.output.status.success(),
        "{}",
        String::from_utf8_lossy(&transcript.output.stderr)
    );
}
