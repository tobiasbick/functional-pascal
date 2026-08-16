//! JSONL protocol stdout stays framed; live debuggee stdin is rejected.

#![allow(
    clippy::expect_used,
    reason = "protocol tests use expect to keep fixture failures local"
)]

use std::io::Cursor;

use fpas_debug::{
    PreparedDebugTarget,
    jsonl::{JsonlServer, ServerStatus, serve, serve_script},
};
use serde_json::{Value, json};

const SOURCE: &str = r#"program TransportOutput;

uses Std.Console;

begin
  WriteLn('hello-raw')
end.
"#;

fn server() -> JsonlServer {
    let (program, diagnostics) = fpas_parser::parse(SOURCE);
    assert!(diagnostics.is_empty(), "parse diagnostics: {diagnostics:?}");
    let executable = fpas_compiler::compile(&program).expect("compile transport fixture");
    JsonlServer::new(PreparedDebugTarget::new(executable, Vec::new())).expect("JSONL server")
}

fn request(id: u64, command: &str, arguments: Value) -> String {
    json!({"type":"request","id":id,"command":command,"arguments":arguments}).to_string()
}

fn parse_records(output: &[u8]) -> Vec<Value> {
    String::from_utf8(output.to_vec())
        .expect("UTF-8 protocol stdout")
        .lines()
        .map(|line| serde_json::from_str::<Value>(line).expect("JSONL protocol line"))
        .collect()
}

#[test]
fn initialize_advertises_structured_output_without_live_input() {
    let mut server = server();
    let initialized = server.handle_line(&request(1, "initialize", json!({"version":2})));
    let capabilities = &initialized[0]["body"]["capabilities"];
    assert_eq!(capabilities["structured_output"], true);
    assert_eq!(capabilities["live_input"], false);
    assert_eq!(capabilities["live_terminal"], false);
}

#[test]
fn program_output_is_structured_events_not_raw_protocol_bytes() {
    let script = [
        request(1, "initialize", json!({"version":2})),
        request(2, "launch", json!({"stop_on_entry":false})),
    ]
    .join("\n");
    let mut output = Vec::new();
    serve_script(script.as_bytes(), &mut output, server()).expect("serve script");
    assert!(
        !output
            .windows(b"hello-raw\n".len())
            .any(|window| window == b"hello-raw\n"),
        "raw debuggee bytes leaked onto protocol stdout: {}",
        String::from_utf8_lossy(&output)
    );
    let records = parse_records(&output);
    let texts: Vec<&str> = records
        .iter()
        .filter(|record| record["event"] == "output")
        .filter_map(|record| record["body"]["text"].as_str())
        .collect();
    assert_eq!(texts, ["hello-raw\n"]);
    assert!(records.iter().all(|record| record.is_object()));
}

#[test]
fn raw_stdin_is_a_protocol_error_not_debuggee_input() {
    let mut server = server();
    let _ = server.handle_line(&request(1, "initialize", json!({"version":2})));
    let records = server.handle_line("hello-raw");
    assert_eq!(records[0]["event"], "protocol_error");
    assert_eq!(records[0]["body"]["code"], "invalid_request");
    assert_eq!(server.status(), ServerStatus::Terminated);
}

#[test]
fn live_input_command_rejects_without_launching_output() {
    let mut server = server();
    let _ = server.handle_line(&request(1, "initialize", json!({"version":2})));
    let _ = server.handle_line(&request(2, "launch", json!({"stop_on_entry":true})));
    assert_eq!(server.status(), ServerStatus::Stopped);
    let rejected = server.handle_line(&request(3, "io.input", json!({"text":"hello-raw"})));
    assert_eq!(rejected[0]["success"], false, "{rejected:?}");
    assert_eq!(rejected[0]["error"]["code"], "live_input_unsupported");
    assert_eq!(server.status(), ServerStatus::Stopped);
    let disconnected = server.handle_line(&request(4, "disconnect", json!({})));
    assert!(disconnected.iter().any(|record| record["success"] == true));
    assert!(
        !disconnected.iter().any(|record| {
            record["event"] == "output" && record["body"]["text"] == "hello-raw\n"
        })
    );
}

#[test]
fn protocol_stdin_eof_does_not_inject_debuggee_bytes() {
    let script = format!(
        "{}\n{}\n",
        request(1, "initialize", json!({"version":2})),
        request(2, "launch", json!({"stop_on_entry":true}))
    );
    let mut output = Vec::new();
    serve(Cursor::new(script.into_bytes()), &mut output, server()).expect("serve until EOF");
    let records = parse_records(&output);
    assert!(
        records
            .iter()
            .any(|record| record["event"] == "stopped" && record["body"]["reason"] == "entry")
    );
    assert!(!records.iter().any(|record| record["event"] == "output"));
}
