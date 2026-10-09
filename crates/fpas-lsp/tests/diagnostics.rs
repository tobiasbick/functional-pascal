//! Protocol-level integration tests for LSP diagnostics and document updates.

#![allow(
    clippy::expect_used,
    clippy::panic,
    reason = "protocol fixtures use explicit assertions for hard-coded LSP transcripts"
)]

mod support;

#[path = "diagnostics/project_errors.rs"]
mod project_errors;
#[path = "diagnostics/project_sources.rs"]
mod project_sources;
#[path = "diagnostics/root_imports.rs"]
mod root_imports;

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
    assert_eq!(diagnostics[0]["code"], json!("FP1013"));
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
        TranscriptStep::Message(open(
            uri,
            1,
            "program Broken;\nbegin\n  if then; end if;\nend.\n",
        )),
        TranscriptStep::Wait(ANALYSIS_WAIT),
        TranscriptStep::Message(change(
            uri,
            2,
            "program Semantic;\nbegin\n  const Value: integer := 'wrong';\nend.\n",
        )),
        TranscriptStep::Wait(ANALYSIS_WAIT),
        TranscriptStep::Message(change(
            uri,
            3,
            "program Fixed;\nbegin\n  const Value: integer := 1;\nend.\n",
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
                .is_some_and(|code| code.starts_with("FP2"))),
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
                    .is_some_and(|code| code.starts_with("FP3"))
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
        TranscriptStep::Message(open(
            uri,
            1,
            "program First;\nbegin\n  if then; end if;\nend.\n",
        )),
        TranscriptStep::Message(change(
            uri,
            2,
            "program Second;\nbegin\n  const Value: integer := 'wrong';\nend.\n",
        )),
        TranscriptStep::Message(change(
            uri,
            3,
            "program Latest;\nbegin\n  const Value: integer := 1;\nend.\n",
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
fn close_during_debounce_cancels_analysis_and_clears_diagnostics() {
    let uri = "file:///phase5/closed.fpas";
    let transcript = run_script(&[
        TranscriptStep::Message(initialize(1)),
        TranscriptStep::Message(initialized()),
        TranscriptStep::Message(open(
            uri,
            1,
            "program Closed;\nbegin\n  if then; end if;\nend.\n",
        )),
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
