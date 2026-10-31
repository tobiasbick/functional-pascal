//! Actual LSP transport ordering and editor-version ownership for the unit view.

#![allow(
    clippy::expect_used,
    reason = "protocol assertions use fixed transcripts"
)]
use crate::support;

use serde_json::json;
use support::{TempDirectory, exit, initialize_with_root, initialized, response, run, shutdown};

#[test]
fn custom_view_request_follows_changes_and_rejects_closed_documents() {
    let temp = TempDirectory::new("public-signatures");
    let uri = temp.uri("unit.fpas");
    let source = "unit Demo;\npublic const First: integer := 1;\nend unit;\n";
    temp.write("unit.fpas", source);
    let transcript = run(&[
        initialize_with_root(1, Some(&temp.uri("."))),
        initialized(),
        json!({"jsonrpc":"2.0","method":"textDocument/didOpen","params":{"textDocument":{"uri":uri,"languageId":"fpas","version":1,"text":source}}}),
        json!({"jsonrpc":"2.0","method":"textDocument/didChange","params":{"textDocument":{"uri":uri,"version":2},"contentChanges":[{"text":source.replace("First", "Changed")}]}}),
        json!({"jsonrpc":"2.0","id":2,"method":"fpas/publicSignatures","params":{"uri":uri}}),
        json!({"jsonrpc":"2.0","method":"textDocument/didClose","params":{"textDocument":{"uri":uri}}}),
        json!({"jsonrpc":"2.0","id":3,"method":"fpas/publicSignatures","params":{"uri":uri}}),
        shutdown(4),
        exit(),
    ]);
    assert!(transcript.output.status.success());
    let result = &response(&transcript.messages, 2)["result"];
    assert_eq!(result["version"], 2);
    assert_eq!(
        result["text"],
        "unit Demo;\n\npublic const Changed: integer;\n\nend unit;\n"
    );
    assert!(response(&transcript.messages, 3).get("error").is_some());
    assert_eq!(
        std::fs::read_to_string(temp.path().join("unit.fpas")).expect("unchanged disk"),
        source
    );
}
