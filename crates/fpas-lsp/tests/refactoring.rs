//! User-visible refactoring edits over the real LSP transport.

#![allow(
    clippy::expect_used,
    clippy::panic,
    reason = "protocol fixtures assert source edits"
)]

use crate::support;

use serde_json::{Value, json};
use support::{
    TempDirectory, TranscriptStep, exit, initialize_with_root, initialized, response, run_script,
    shutdown,
};

#[test]
fn extract_variable_is_a_versioned_refactoring_without_diagnostics() {
    let temp = TempDirectory::new("extract-variable");
    let source = "program Extract;\nbegin\n  const Output := (1 + 2) * 3;\nend.\n";
    temp.write("main.fpas", source);
    let uri = temp.uri("main.fpas");
    let start = source.find("(1 + 2)").expect("selected expression");
    let transcript = run_script(&[
        TranscriptStep::Message(initialize_with_root(1, Some(&temp.uri(".")))),
        TranscriptStep::Message(initialized()),
        TranscriptStep::Message(
            json!({"jsonrpc":"2.0", "method":"textDocument/didOpen", "params": {
                "textDocument": {"uri":uri, "languageId":"fpas", "version":7, "text":source}
            }}),
        ),
        TranscriptStep::Message(
            json!({"jsonrpc":"2.0", "id":2, "method":"textDocument/codeAction", "params": {
                "textDocument":{"uri":uri}, "range":{"start":position(source,start),"end":position(source,start+7)},
                "context":{"diagnostics":[],"only":["refactor.extract"]}
            }}),
        ),
        TranscriptStep::Message(
            json!({"jsonrpc":"2.0","id":5,"method":"fpas/refactor","params":{
                "version":1,"phase":"prepare","recipe":{"action":"extract-variable","uri":uri,"range":{"start":position(source,start),"end":position(source,start+7)}}
            }}),
        ),
        TranscriptStep::MessageFrom(Box::new(|messages| {
            json!({"jsonrpc":"2.0", "id":3,"method":"workspace/executeCommand","params":{
                "command":"fpas.applyRefactor", "arguments":response(messages,2)["result"][0]["command"]["arguments"]
            }})
        })),
        TranscriptStep::Message(shutdown(4)),
        TranscriptStep::Message(exit()),
    ]);
    assert!(
        transcript.output.status.success(),
        "{}",
        String::from_utf8_lossy(&transcript.output.stderr)
    );
    let actions = response(&transcript.messages, 2)["result"]
        .as_array()
        .expect("actions");
    let [action] = actions.as_slice() else {
        panic!(
            "expected extract refactoring: {actions:?}; direct preparation: {}",
            response(&transcript.messages, 5)
        )
    };
    assert_eq!(action["kind"], "refactor.extract");
    assert!(
        action.get("edit").is_none(),
        "actions carry no cached edits"
    );
    assert_eq!(
        response(&transcript.messages, 3)["result"]["status"],
        "applied"
    );
    let apply = transcript
        .messages
        .iter()
        .find(|m| m["method"] == "workspace/applyEdit")
        .expect("editor application request");
    let edit = &apply["params"]["edit"]["documentChanges"][0];
    assert_eq!(edit["textDocument"], json!({"uri":uri,"version":7}));
    assert_eq!(
        edit["edits"],
        json!([
            {"range":{"start":{"line":2,"character":0},"end":{"line":2,"character":0}},
             "newText":"  const ExtractedValue := (1 + 2);\n"},
            {"range":{"start":position(source,start),"end":position(source,start+7)},"newText":"ExtractedValue"}
        ])
    );
}

fn position(source: &str, offset: usize) -> Value {
    let before = &source[..offset];
    let line = before.bytes().filter(|byte| *byte == b'\n').count();
    let start = before.rfind('\n').map_or(0, |index| index + 1);
    json!({"line":line,"character":source[start..offset].encode_utf16().count()})
}
