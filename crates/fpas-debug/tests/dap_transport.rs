//! DAP framing keeps protocol bytes distinct from queued debuggee input.

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

const TERMINAL_SOURCE: &str = r#"program TerminalInput;

uses Std.Console;

begin
  AcquireInteractiveTerminal();
  WriteLn('terminal-ready');
  while not EventPending() do
  begin
  end;
  var InputEvent: ConsoleEvent := ReadEvent();
  WriteLn(InputEvent.key.ch);
  if KeyPressed() then
  begin
    WriteLn('duplicate')
  end
  else
  begin
    WriteLn('clean')
  end;
  ReleaseInteractiveTerminal()
end.
"#;

fn target(source: &'static str) -> PreparedDebugTarget {
    let (program, diagnostics) = fpas_parser::parse(source);
    assert!(diagnostics.is_empty(), "parse diagnostics: {diagnostics:?}");
    let executable = fpas_compiler::compile(&program).expect("compile DAP transport fixture");
    PreparedDebugTarget::new(executable, Vec::new()).with_sources(vec![DebugSourceContent {
        path: "<memory>".into(),
        original_path: None,
        content: source.into(),
    }])
}

fn server() -> DapServer {
    DapServer::new(target(OUTPUT_SOURCE)).expect("DAP server")
}

fn input_server() -> DapServer {
    DapServer::new(target(INPUT_SOURCE)).expect("DAP input server")
}

fn terminal_server() -> DapServer {
    DapServer::new(target(TERMINAL_SOURCE)).expect("DAP terminal server")
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

fn stop_on_entry(adapter: &mut DapServer) {
    let _ = adapter.handle(request(1, "initialize", json!({})));
    let _ = adapter.handle(request(2, "launch", json!({"stopOnEntry":true})));
    let configured = adapter.handle(request(3, "configurationDone", json!({})));
    assert!(
        configured
            .iter()
            .any(|message| message["event"] == "stopped")
    );
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
fn terminal_events_reach_a_running_tui_and_output_is_streamed() {
    let mut adapter = terminal_server();
    let _ = adapter.handle(request(1, "initialize", json!({})));
    let _ = adapter.handle(request(2, "launch", json!({"stopOnEntry":false})));
    let mut messages = adapter.handle(request(3, "configurationDone", json!({})));

    let mut sequence = 4;
    for _ in 0..1_000 {
        let polled = adapter.handle(request(sequence, "fpas/terminalPoll", json!({})));
        sequence += 1;
        let received_terminal_output = polled
            .iter()
            .any(|message| message["event"] == "fpas/terminalOutput");
        messages.extend(polled);
        if received_terminal_output {
            break;
        }
        std::thread::sleep(std::time::Duration::from_millis(1));
    }
    assert!(
        messages
            .iter()
            .any(|message| message["event"] == "fpas/terminalOutput"),
        "running terminal output was not pollable"
    );

    let accepted = adapter.handle(request(
        sequence,
        "fpas/terminalInput",
        json!({"events":[{"kind":"key","key":"Character","text":"q"}]}),
    ));
    assert!(
        accepted
            .iter()
            .any(|message| message["success"] == true && message["command"] == "fpas/terminalInput"),
        "terminal input response: {accepted:?}"
    );
    messages.extend(accepted);
    messages.extend(adapter.wait());

    let terminal_output = messages
        .iter()
        .filter(|message| message["event"] == "fpas/terminalOutput")
        .filter_map(|message| message["body"]["output"].as_str())
        .collect::<String>();
    assert!(
        terminal_output.contains("\u{1b}[?1049h")
            && terminal_output.contains("\u{1b}[?1049l")
            && terminal_output.contains('q'),
        "terminal stream contained {} bytes",
        terminal_output.len()
    );
    let logical_output = messages
        .iter()
        .filter(|message| message["event"] == "output")
        .filter_map(|message| message["body"]["output"].as_str())
        .collect::<String>();
    assert_eq!(logical_output, "terminal-ready\nq\nclean\n");
    assert!(
        messages
            .iter()
            .any(|message| message["event"] == "terminated"),
        "{messages:?}"
    );
}

#[test]
fn live_input_request_matches_jsonl_and_stays_structured() {
    let mut adapter = input_server();
    stop_on_entry(&mut adapter);
    let accepted = adapter.handle(request(4, "fpas/input", json!({"text":"one"})));
    assert_eq!(accepted[0]["success"], true, "{accepted:?}");
    assert_eq!(accepted[0]["body"]["bytes"], 4);
    assert_eq!(accepted[0]["body"]["sessionBytes"], 4);
    let _ = adapter.handle(request(5, "fpas/input", json!({"text":"two"})));
    let continued = adapter.handle(request(6, "continue", json!({})));
    let mut records = continued;
    records.extend(adapter.wait());
    let texts: Vec<&str> = records
        .iter()
        .filter(|message| message["event"] == "output")
        .filter_map(|message| message["body"]["output"].as_str())
        .collect();
    assert_eq!(texts, ["one\n", "two\n"]);

    let mut jsonl = JsonlServer::new(target(INPUT_SOURCE)).expect("JSONL server");
    let _ = jsonl.handle_line(&jsonl_request(1, "initialize", json!({"version":2})));
    let _ = jsonl.handle_line(&jsonl_request(2, "launch", json!({"stop_on_entry":true})));
    let accepted = jsonl.handle_line(&jsonl_request(3, "io.input", json!({"text":"one"})));
    assert_eq!(accepted[0]["body"]["bytes"], 4);
}

#[test]
fn eof_request_rejects_further_input() {
    let mut adapter = input_server();
    stop_on_entry(&mut adapter);
    let eof = adapter.handle(request(4, "fpas/eof", json!({})));
    assert_eq!(eof[0]["success"], true, "{eof:?}");
    assert_eq!(eof[0]["body"]["eof"], true);
    let late = adapter.handle(request(5, "fpas/input", json!({"text":"late"})));
    assert_eq!(late[0]["success"], false, "{late:?}");
    assert_eq!(late[0]["body"]["error"]["code"], "debuggee_input_closed");
}

#[test]
fn cancel_input_clears_unread_lines() {
    let mut adapter = input_server();
    stop_on_entry(&mut adapter);
    let _ = adapter.handle(request(4, "fpas/input", json!({"text":"secret"})));
    let cleared = adapter.handle(request(5, "fpas/cancelInput", json!({})));
    assert_eq!(cleared[0]["success"], true, "{cleared:?}");
    assert_eq!(cleared[0]["body"]["cleared"], true);
}

#[test]
fn tui_event_requests_are_unsupported() {
    let mut adapter = server();
    stop_on_entry(&mut adapter);
    let tui = adapter.handle(request(4, "fpas/event", json!({})));
    assert_eq!(tui[0]["success"], false, "{tui:?}");
}
