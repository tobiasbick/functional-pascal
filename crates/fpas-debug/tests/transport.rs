//! JSONL protocol stdout stays framed; queued debuggee input is ordered.

#![allow(
    clippy::expect_used,
    reason = "protocol tests use expect to keep fixture failures local"
)]

mod support;

use std::io::Cursor;
use std::sync::atomic::Ordering;

use fpas_debug::{
    PreparedDebugTarget,
    jsonl::{JsonlServer, ServerStatus, serve, serve_script},
};
use fpas_vm::DebugExecutionLimits;
use serde_json::{Value, json};

const OUTPUT_SOURCE: &str = r#"program TransportOutput;

uses Std.Console;

begin
  WriteLn('hello-raw')
end.
"#;

const INPUT_SOURCE: &str = r#"program TransportInput;

uses Std.Console;

begin
  WriteLn(ReadLn());
  WriteLn(ReadLn())
end.
"#;

fn server_for(source: &str) -> JsonlServer {
    let (program, diagnostics) = fpas_parser::parse(source);
    assert!(diagnostics.is_empty(), "parse diagnostics: {diagnostics:?}");
    let executable = fpas_compiler::compile(&program).expect("compile transport fixture");
    JsonlServer::new(PreparedDebugTarget::new(executable, Vec::new())).expect("JSONL server")
}

fn server() -> JsonlServer {
    server_for(OUTPUT_SOURCE)
}

fn input_server() -> JsonlServer {
    server_for(INPUT_SOURCE)
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

fn output_texts(records: &[Value]) -> Vec<&str> {
    records
        .iter()
        .filter(|record| record["event"] == "output")
        .filter_map(|record| record["body"]["text"].as_str())
        .collect()
}

#[test]
fn initialize_advertises_structured_output_and_live_input() {
    let mut server = server();
    let initialized = server.handle_line(&request(1, "initialize", json!({"version":2})));
    let capabilities = &initialized[0]["body"]["capabilities"];
    assert_eq!(capabilities["structured_output"], true);
    assert_eq!(capabilities["live_input"], true);
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
    assert_eq!(output_texts(&records), ["hello-raw\n"]);
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
fn malformed_script_returns_transport_error_after_protocol_record() {
    let mut output = Vec::new();
    let error = serve_script(b"\n{\n".as_slice(), &mut output, server())
        .expect_err("malformed JSONL must fail the scripted transport");
    assert_eq!(error.kind(), std::io::ErrorKind::InvalidData);
    let records = parse_records(&output);
    assert_eq!(records.len(), 1);
    assert_eq!(records[0]["event"], "protocol_error");
}

#[test]
fn malformed_live_input_returns_transport_error_after_protocol_record() {
    let mut output = Vec::new();
    let error = serve(Cursor::new(b"{\n"), &mut output, server())
        .expect_err("malformed JSONL must fail the live transport");
    assert_eq!(error.kind(), std::io::ErrorKind::InvalidData);
    let records = parse_records(&output);
    assert_eq!(records[0]["event"], "protocol_error");
}

#[test]
fn clean_disconnect_remains_successful() {
    let script = [
        request(1, "initialize", json!({"version":2})),
        request(2, "disconnect", json!({})),
    ]
    .join("\n");
    serve_script(script.as_bytes(), Vec::new(), server()).expect("clean disconnect");
}

#[test]
fn live_disconnect_drops_an_input_that_stays_open() {
    let script = [
        request(1, "initialize", json!({"version":2})),
        request(2, "disconnect", json!({})),
    ]
    .join("\n");
    let (reader, dropped, _keep_open) = support::open_reader(format!("{script}\n").into_bytes());

    serve(reader, Vec::new(), server()).expect("disconnect open JSONL input");

    assert!(
        dropped.load(Ordering::Acquire),
        "JSONL reader remained owned by a detached thread"
    );
}

#[test]
fn jsonl_transport_rejects_oversized_lines_with_and_without_newline() {
    const MAX_LINE_BYTES: usize = 16 * 1024 * 1024;

    for newline in [false, true] {
        let mut input = vec![b' '; MAX_LINE_BYTES + 1];
        if newline {
            input.push(b'\n');
        }
        let error = serve_script(input.as_slice(), Vec::new(), server())
            .expect_err("oversized JSONL line must fail");
        assert_eq!(error.kind(), std::io::ErrorKind::InvalidData);
        assert!(error.to_string().contains("16 MiB"), "{error}");
    }
}

#[test]
fn jsonl_transport_accepts_a_line_at_the_exact_limit() {
    const MAX_LINE_BYTES: usize = 16 * 1024 * 1024;
    const PREFIX: &str =
        r#"{"type":"request","id":1,"command":"initialize","arguments":{},"padding":""#;
    const SUFFIX: &str = "\"}\n";

    let padding_len = MAX_LINE_BYTES - PREFIX.len() - SUFFIX.len();
    let mut input = Vec::with_capacity(MAX_LINE_BYTES);
    input.extend_from_slice(PREFIX.as_bytes());
    input.resize(input.len() + padding_len, b'x');
    input.extend_from_slice(SUFFIX.as_bytes());
    assert_eq!(input.len(), MAX_LINE_BYTES);

    serve_script(input.as_slice(), Vec::new(), server())
        .expect("a JSONL request at the exact line limit must parse");
}

#[test]
fn queued_input_is_ordered_and_does_not_use_protocol_stdin() {
    let mut server = input_server();
    let _ = server.handle_line(&request(1, "initialize", json!({"version":2})));
    let _ = server.handle_line(&request(2, "launch", json!({"stop_on_entry":true})));
    assert_eq!(server.status(), ServerStatus::Stopped);
    let first = server.handle_line(&request(3, "io.input", json!({"text":"one"})));
    assert_eq!(first[0]["success"], true, "{first:?}");
    assert_eq!(first[0]["body"]["bytes"], 4);
    let second = server.handle_line(&request(4, "io.input", json!({"text":"two"})));
    assert_eq!(second[0]["body"]["session_bytes"], 8, "{second:?}");
    let missing = server.handle_line(&request(5, "io.input", json!({})));
    assert_eq!(missing[0]["error"]["code"], "invalid_request");
    let mut records = server.handle_line(&request(6, "continue", json!({})));
    records.extend(server.wait());
    assert_eq!(output_texts(&records), ["one\n", "two\n"]);
}

#[test]
fn eof_and_cancel_are_deterministic() {
    let mut eof_server = input_server();
    let _ = eof_server.handle_line(&request(1, "initialize", json!({"version":2})));
    let _ = eof_server.handle_line(&request(2, "launch", json!({"stop_on_entry":true})));
    let eof = eof_server.handle_line(&request(3, "io.eof", json!({})));
    assert_eq!(eof[0]["body"]["eof"], true, "{eof:?}");
    let late = eof_server.handle_line(&request(4, "io.input", json!({"text":"late"})));
    assert_eq!(late[0]["error"]["code"], "debuggee_input_closed");
    let mut records = eof_server.handle_line(&request(5, "continue", json!({})));
    records.extend(eof_server.wait());
    assert!(
        records.iter().any(|record| {
            record["event"] == "stopped" && record["body"]["reason"] == "runtime_error"
        }),
        "{records:?}"
    );

    let mut cancel_server = input_server();
    let _ = cancel_server.handle_line(&request(1, "initialize", json!({"version":2})));
    let _ = cancel_server.handle_line(&request(2, "launch", json!({"stop_on_entry":true})));
    let _ = cancel_server.handle_line(&request(3, "io.input", json!({"text":"secret"})));
    let cleared = cancel_server.handle_line(&request(4, "io.cancel", json!({})));
    assert_eq!(cleared[0]["body"]["cleared"], true, "{cleared:?}");
    let mut records = cancel_server.handle_line(&request(5, "continue", json!({})));
    records.extend(cancel_server.wait());
    assert!(output_texts(&records).is_empty(), "{records:?}");
}

#[test]
fn input_limit_is_a_stable_error() {
    let (program, diagnostics) = fpas_parser::parse(INPUT_SOURCE);
    assert!(diagnostics.is_empty(), "parse diagnostics: {diagnostics:?}");
    let executable = fpas_compiler::compile(&program).expect("compile limit fixture");
    let mut server = JsonlServer::new(
        PreparedDebugTarget::new(executable, Vec::new()).with_execution_limits(
            DebugExecutionLimits {
                max_input_bytes: 1,
                ..DebugExecutionLimits::default()
            },
        ),
    )
    .expect("limited JSONL server");
    let _ = server.handle_line(&request(1, "initialize", json!({"version":2})));
    let _ = server.handle_line(&request(2, "launch", json!({"stop_on_entry":true})));
    let rejected = server.handle_line(&request(3, "io.input", json!({"text":"ab"})));
    assert_eq!(rejected[0]["error"]["code"], "debuggee_input_limit");
    assert_eq!(server.status(), ServerStatus::Stopped);
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

#[test]
fn tui_event_commands_are_unsupported() {
    let mut server = server();
    let _ = server.handle_line(&request(1, "initialize", json!({"version":2})));
    let _ = server.handle_line(&request(2, "launch", json!({"stop_on_entry":true})));
    let tui = server.handle_line(&request(3, "io.event", json!({"text":"x"})));
    assert_eq!(tui[0]["error"]["code"], "unsupported_capability");
    assert_eq!(server.status(), ServerStatus::Stopped);
}
