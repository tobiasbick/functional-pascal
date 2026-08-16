//! DAP framing keeps protocol bytes distinct from captured debuggee output.

#![allow(
    clippy::expect_used,
    reason = "DAP transcript fixtures use direct assertions"
)]

use std::io::{BufReader, Cursor};

use fpas_debug::{
    DebugSourceContent, PreparedDebugTarget,
    dap::{DapServer, read_message, serve, write_message},
    jsonl::JsonlServer,
};
use serde_json::{Value, json};

const SOURCE: &str = r#"program TransportOutput;

uses Std.Console;

begin
  WriteLn('hello-raw')
end.
"#;

fn target() -> PreparedDebugTarget {
    let (program, diagnostics) = fpas_parser::parse(SOURCE);
    assert!(diagnostics.is_empty(), "parse diagnostics: {diagnostics:?}");
    let executable = fpas_compiler::compile(&program).expect("compile DAP transport fixture");
    PreparedDebugTarget::new(executable, Vec::new()).with_sources(vec![DebugSourceContent {
        path: "<memory>".into(),
        content: SOURCE.into(),
    }])
}

fn server() -> DapServer {
    DapServer::new(target()).expect("DAP server")
}

fn request(seq: u64, command: &str, arguments: Value) -> Value {
    json!({"seq":seq,"type":"request","command":command,"arguments":arguments})
}

fn jsonl_request(id: u64, command: &str, arguments: Value) -> String {
    json!({"type":"request","id":id,"command":command,"arguments":arguments}).to_string()
}

fn read_all(output: &[u8]) -> Vec<Value> {
    let mut reader = BufReader::new(output);
    let mut messages = Vec::new();
    while let Some(message) = read_message(&mut reader).expect("framed DAP message") {
        messages.push(message);
    }
    messages
}

#[test]
fn unframed_stdin_is_a_framing_error_not_debuggee_input() {
    let error = read_message(&mut BufReader::new(Cursor::new(b"hello-raw\n")))
        .expect_err("raw debuggee bytes are not DAP");
    assert_eq!(error.kind(), std::io::ErrorKind::InvalidData);
}

#[test]
fn program_output_is_a_framed_event_not_raw_protocol_bytes() {
    let requests = [
        request(1, "initialize", json!({})),
        request(2, "launch", json!({"stopOnEntry":false})),
        request(3, "configurationDone", json!({})),
    ];
    let mut input = Vec::new();
    for request in requests {
        write_message(&mut input, &request).expect("frame request");
    }
    let mut output = Vec::new();
    serve(Cursor::new(input), &mut output, server()).expect("serve DAP");
    assert!(
        output.starts_with(b"Content-Length:"),
        "DAP stdout must stay framed: {}",
        String::from_utf8_lossy(&output)
    );
    let messages = read_all(&output);
    let texts: Vec<&str> = messages
        .iter()
        .filter(|message| message["event"] == "output")
        .filter_map(|message| message["body"]["output"].as_str())
        .collect();
    assert_eq!(texts, ["hello-raw\n"]);
    assert!(
        messages
            .iter()
            .any(|message| message["event"] == "terminated")
    );
}

#[test]
fn live_input_request_rejects_and_matches_jsonl() {
    let mut adapter = server();
    let _ = adapter.handle(request(1, "initialize", json!({})));
    let _ = adapter.handle(request(2, "launch", json!({"stopOnEntry":true})));
    let configured = adapter.handle(request(3, "configurationDone", json!({})));
    assert!(
        configured
            .iter()
            .any(|message| message["event"] == "stopped")
    );

    let rejected = adapter.handle(request(4, "fpas/input", json!({"text":"hello-raw"})));
    assert_eq!(rejected[0]["success"], false, "{rejected:?}");
    assert_eq!(
        rejected[0]["body"]["error"]["code"],
        "live_input_unsupported"
    );

    let mut jsonl = JsonlServer::new(target()).expect("JSONL server");
    let _ = jsonl.handle_line(&jsonl_request(1, "initialize", json!({"version":2})));
    let _ = jsonl.handle_line(&jsonl_request(3, "launch", json!({"stop_on_entry":true})));
    let rejected = jsonl.handle_line(&jsonl_request(4, "io.input", json!({"text":"hello-raw"})));
    assert_eq!(rejected[0]["error"]["code"], "live_input_unsupported");
}
