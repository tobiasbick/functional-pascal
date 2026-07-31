#![allow(
    clippy::expect_used,
    clippy::panic,
    reason = "process fixtures use explicit panics to report malformed hard-coded transcripts"
)]

mod support;

#[path = "support/raw.rs"]
mod raw;

use serde_json::json;

use raw::{frame_bytes, run_frames};
use support::{exit, initialize, initialized, response, run, shutdown};

#[test]
fn stdio_transcript_supports_initialize_documents_shutdown_and_exit() {
    let uri = "file:///phase4/lifecycle.fpas";
    let transcript = run(&[
        initialize(1),
        initialized(),
        json!({
            "jsonrpc": "2.0",
            "method": "textDocument/didOpen",
            "params": {
                "textDocument": {
                    "uri": uri,
                    "languageId": "fpas",
                    "version": 1,
                    "text": "program Lifecycle;\nbegin\nend.\n"
                }
            }
        }),
        json!({
            "jsonrpc": "2.0",
            "method": "textDocument/didChange",
            "params": {
                "textDocument": {"uri": uri, "version": 2},
                "contentChanges": [
                    {"text": "program Lifecycle;\nbegin\n  WriteLn('ok');\nend.\n"}
                ]
            }
        }),
        json!({
            "jsonrpc": "2.0",
            "method": "textDocument/didSave",
            "params": {"textDocument": {"uri": uri}}
        }),
        json!({
            "jsonrpc": "2.0",
            "method": "textDocument/didClose",
            "params": {"textDocument": {"uri": uri}}
        }),
        shutdown(2),
        exit(),
    ]);

    assert!(
        transcript.output.status.success(),
        "{}",
        String::from_utf8_lossy(&transcript.output.stderr)
    );
    assert!(
        transcript.output.stdout.is_empty(),
        "stdout contained bytes outside the parsed LSP response frames"
    );
    let initialize = response(&transcript.messages, 1);
    let capabilities = initialize["result"]["capabilities"]
        .as_object()
        .expect("initialize capabilities object");
    let mut capability_names = capabilities.keys().map(String::as_str).collect::<Vec<_>>();
    capability_names.sort_unstable();
    assert_eq!(
        capability_names,
        [
            "completionProvider",
            "definitionProvider",
            "documentFormattingProvider",
            "documentSymbolProvider",
            "hoverProvider",
            "positionEncoding",
            "referencesProvider",
            "renameProvider",
            "textDocumentSync"
        ]
    );
    assert_eq!(
        capabilities["completionProvider"]["triggerCharacters"],
        json!(["."])
    );
    assert_eq!(capabilities["definitionProvider"], json!(true));
    assert_eq!(capabilities["documentFormattingProvider"], json!(true));
    assert_eq!(capabilities["documentSymbolProvider"], json!(true));
    assert_eq!(capabilities["hoverProvider"], json!(true));
    assert_eq!(capabilities["referencesProvider"], json!(true));
    assert_eq!(
        capabilities["renameProvider"]["prepareProvider"],
        json!(true)
    );
    assert_eq!(
        capabilities["positionEncoding"],
        json!("utf-16"),
        "{initialize:?}"
    );
    assert_eq!(capabilities["textDocumentSync"]["openClose"], json!(true));
    assert_eq!(capabilities["textDocumentSync"]["change"], json!(1));
    assert_eq!(
        capabilities["textDocumentSync"]["save"]["includeText"],
        json!(false)
    );
    assert_eq!(
        response(&transcript.messages, 2)["result"],
        serde_json::Value::Null
    );
}

#[test]
fn invalid_uri_stale_version_incremental_change_and_cancel_do_not_crash() {
    let uri = "file:///phase4/invalid-input.fpas";
    let transcript = run(&[
        initialize(1),
        initialized(),
        json!({
            "jsonrpc": "2.0",
            "method": "textDocument/didOpen",
            "params": {
                "textDocument": {
                    "uri": uri,
                    "languageId": "fpas",
                    "version": 2,
                    "text": "program Input;\nbegin\nend.\n"
                }
            }
        }),
        json!({
            "jsonrpc": "2.0",
            "method": "textDocument/didChange",
            "params": {
                "textDocument": {"uri": uri, "version": 1},
                "contentChanges": [{"text": "program Stale;\nbegin\nend.\n"}]
            }
        }),
        json!({
            "jsonrpc": "2.0",
            "method": "textDocument/didChange",
            "params": {
                "textDocument": {"uri": uri, "version": 3},
                "contentChanges": [{
                    "range": {
                        "start": {"line": 0, "character": 0},
                        "end": {"line": 0, "character": 0}
                    },
                    "text": "x"
                }]
            }
        }),
        json!({
            "jsonrpc": "2.0",
            "method": "textDocument/didOpen",
            "params": {
                "textDocument": {
                    "uri": "untitled:Functional-Pascal-1",
                    "languageId": "fpas",
                    "version": 1,
                    "text": ""
                }
            }
        }),
        json!({
            "jsonrpc": "2.0",
            "method": "$/cancelRequest",
            "params": {"id": 999}
        }),
        shutdown(2),
        exit(),
    ]);

    assert!(transcript.output.status.success());
    assert!(
        transcript.output.stdout.is_empty(),
        "stdout contained bytes outside the parsed LSP response frames"
    );
    assert_eq!(
        response(&transcript.messages, 2)["result"],
        serde_json::Value::Null
    );
    let stderr = String::from_utf8_lossy(&transcript.output.stderr);
    assert!(stderr.contains("Stale document version"), "{stderr}");
    assert!(
        stderr.contains("Incremental text changes are unsupported"),
        "{stderr}"
    );
    assert!(stderr.contains("unsupported URI scheme"), "{stderr}");
}

#[test]
fn malformed_json_receives_a_framed_parse_error() {
    let frames = [frame_bytes(b"{")];
    let transcript = run_frames(&frames);

    assert!(transcript.output.status.success());
    assert!(transcript.messages.iter().any(|message| {
        message["error"]["code"] == json!(-32700) && message["id"] == serde_json::Value::Null
    }));
}
