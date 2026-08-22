//! Protocol tests for LSP document and range formatting.

#![allow(
    clippy::expect_used,
    clippy::panic,
    reason = "protocol fixtures use explicit assertions for hard-coded LSP transcripts"
)]

mod support;

use serde_json::{Value, json};

use support::{exit, initialize, initialized, response, run, shutdown};

#[test]
fn formatting_matches_the_canonical_fpas_formatter_for_the_unsaved_buffer() {
    let uri = "file:///phase5/format-parity.fpas";
    let source = "program Messy; begin var Value:integer:=1 end.";
    let (unit, diagnostics) = fpas_parser::parse_compilation_unit(source);
    assert!(diagnostics.is_empty(), "{diagnostics:?}");
    let expected = fpas_fmt::format_source(source, &unit).expect("matching source and AST");
    let transcript = run(&[
        initialize(1),
        initialized(),
        open(uri, 1, source),
        formatting_request(2, uri),
        shutdown(3),
        exit(),
    ]);

    assert_success(&transcript);
    let edits = response(&transcript.messages, 2)["result"]
        .as_array()
        .expect("formatting edit array");
    assert_eq!(edits.len(), 1, "{edits:?}");
    assert_eq!(
        edits[0]["range"]["start"],
        json!({"line": 0, "character": 0})
    );
    assert_eq!(
        edits[0]["range"]["end"],
        json!({"line": 0, "character": source.encode_utf16().count()})
    );
    assert_eq!(edits[0]["newText"], json!(expected));
}

#[test]
fn formatting_preserves_comments_and_is_idempotent() {
    let uri = "file:///phase5/format-comments.fpas";
    let source =
        "program Comments; // header\nbegin\n// before\nWriteLn('ok'); // tail\nend. // done\n";
    let (unit, diagnostics) = fpas_parser::parse_compilation_unit(source);
    assert!(diagnostics.is_empty(), "{diagnostics:?}");
    let canonical = fpas_fmt::format_source(source, &unit).expect("matching source and AST");
    for comment in ["// header", "// before", "// tail", "// done"] {
        assert!(canonical.contains(comment), "{canonical}");
    }
    let transcript = run(&[
        initialize(1),
        initialized(),
        open(uri, 1, source),
        formatting_request(2, uri),
        change(uri, 2, &canonical),
        formatting_request(3, uri),
        shutdown(4),
        exit(),
    ]);

    assert_success(&transcript);
    assert_eq!(
        response(&transcript.messages, 2)["result"][0]["newText"],
        json!(canonical)
    );
    assert_eq!(response(&transcript.messages, 3)["result"], json!([]));
}

#[test]
fn malformed_unsaved_input_returns_no_destructive_edit() {
    let uri = "file:///phase5/format-malformed.fpas";
    let transcript = run(&[
        initialize(1),
        initialized(),
        open(uri, 1, "program Broken;\nbegin\n  if then\nend.\n"),
        formatting_request(2, uri),
        shutdown(3),
        exit(),
    ]);

    assert_success(&transcript);
    assert_eq!(response(&transcript.messages, 2)["result"], Value::Null);
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

fn formatting_request(id: i32, uri: &str) -> Value {
    json!({
        "jsonrpc": "2.0",
        "id": id,
        "method": "textDocument/formatting",
        "params": {
            "textDocument": {"uri": uri},
            "options": {"tabSize": 2, "insertSpaces": true}
        }
    })
}

fn assert_success(transcript: &support::Transcript) {
    assert!(
        transcript.output.status.success(),
        "{}",
        String::from_utf8_lossy(&transcript.output.stderr)
    );
}
