//! Protocol tests for opening, updating, and closing LSP documents.

#![allow(
    clippy::expect_used,
    reason = "protocol fixtures use explicit assertions for hard-coded LSP transcripts"
)]

mod support;

use std::time::Duration;

use serde_json::{Value, json};
use support::{TranscriptStep, exit, initialize, initialized, notifications, run_script, shutdown};

#[test]
fn reopened_client_version_publishes_analysis_for_the_new_document_lifetime() {
    let uri = "file:///review/reopened.fpas";
    let transcript = run_script(&[
        TranscriptStep::Message(initialize(1)),
        TranscriptStep::Message(initialized()),
        TranscriptStep::Message(open(uri, 1, "program First;\nbegin\nend.\n")),
        TranscriptStep::Wait(Duration::from_millis(260)),
        TranscriptStep::Message(close(uri)),
        TranscriptStep::Message(open(
            uri,
            1,
            "program Second;\nbegin\n  var Broken: integer := 'text'\nend.\n",
        )),
        TranscriptStep::Wait(Duration::from_millis(260)),
        TranscriptStep::Message(shutdown(2)),
        TranscriptStep::Message(exit()),
    ]);

    assert!(
        transcript.output.status.success(),
        "{}",
        String::from_utf8_lossy(&transcript.output.stderr)
    );
    let version_one = notifications(&transcript.messages, "textDocument/publishDiagnostics")
        .into_iter()
        .filter(|message| message["params"]["version"] == json!(1))
        .collect::<Vec<_>>();
    assert_eq!(version_one.len(), 2, "{version_one:?}");
    assert_eq!(version_one[0]["params"]["diagnostics"], json!([]));
    assert!(
        !version_one[1]["params"]["diagnostics"]
            .as_array()
            .expect("reopened diagnostics")
            .is_empty(),
        "{version_one:?}"
    );
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

fn close(uri: &str) -> Value {
    json!({
        "jsonrpc": "2.0",
        "method": "textDocument/didClose",
        "params": {"textDocument": {"uri": uri}}
    })
}
