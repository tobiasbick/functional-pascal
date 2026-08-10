//! JSONL protocol conformance at the shared debugger boundary.

#![allow(
    clippy::expect_used,
    reason = "protocol tests use expect to keep fixture failures local"
)]

use std::time::Duration;

use fpas_debug::{
    PreparedDebugTarget,
    jsonl::{JsonlServer, ServerStatus, serve_script},
};
use fpas_vm::DebugExecutionLimits;
use serde_json::{Value, json};

fn server(source: &str) -> JsonlServer {
    server_with_limits(source, DebugExecutionLimits::default())
}

fn server_with_limits(source: &str, limits: DebugExecutionLimits) -> JsonlServer {
    let (program, diagnostics) = fpas_parser::parse(source);
    assert!(diagnostics.is_empty(), "parse diagnostics: {diagnostics:?}");
    let executable = fpas_compiler::compile(&program).expect("compile debug fixture");
    JsonlServer::new(PreparedDebugTarget::new(executable, Vec::new()).with_execution_limits(limits))
        .expect("create JSONL server")
}

fn request(id: u64, command: &str, arguments: Value) -> String {
    json!({"type":"request","id":id,"command":command,"arguments":arguments}).to_string()
}

#[test]
fn lifecycle_is_machine_readable_and_deterministic() {
    let script = [
        request(1, "initialize", json!({"version":1})),
        request(2, "launch", json!({"stop_on_entry":true})),
        request(3, "stack", json!({})),
        request(4, "stack", json!({})),
        request(5, "continue", json!({})),
    ]
    .join("\n");
    let mut output = Vec::new();
    serve_script(
        script.as_bytes(),
        &mut output,
        server("program Main; begin var X: integer := 1 end."),
    )
    .expect("serve script");
    let records = String::from_utf8(output).expect("UTF-8 records");
    let values = records
        .lines()
        .map(|line| serde_json::from_str::<Value>(line).expect("valid JSONL"))
        .collect::<Vec<_>>();
    assert!(values.iter().any(|value| value["event"] == "stopped"));
    assert!(values.iter().any(|value| value["event"] == "terminated"));
    let stacks = values
        .iter()
        .filter(|value| value["command"] == "stack")
        .collect::<Vec<_>>();
    assert_eq!(stacks.len(), 2);
    assert_eq!(stacks[0]["body"], stacks[1]["body"]);
}

#[test]
fn malformed_input_and_duplicate_ids_return_stable_errors() {
    let mut malformed = server("program Main; begin end.");
    let records = malformed.handle_line("{");
    assert_eq!(records[0]["body"]["code"], "invalid_request");
    assert_eq!(malformed.status(), ServerStatus::Terminated);

    let mut duplicate = server("program Main; begin end.");
    let _ = duplicate.handle_line(&request(1, "initialize", json!({})));
    let records = duplicate.handle_line(&request(1, "launch", json!({})));
    assert_eq!(records[0]["error"]["code"], "invalid_request");
}

#[test]
fn invalid_state_and_unsupported_commands_are_explicit() {
    let mut server = server("program Main; begin end.");
    let _ = server.handle_line(&request(1, "initialize", json!({})));
    let invalid = server.handle_line(&request(2, "continue", json!({})));
    assert_eq!(invalid[0]["error"]["code"], "invalid_state");
    let unsupported = server.handle_line(&request(3, "evaluate", json!({})));
    assert_eq!(unsupported[0]["error"]["code"], "unsupported_capability");
}

#[test]
fn instruction_and_timeout_limits_are_reported() {
    let cases = [
        (
            "program Main; begin while true do begin end end.",
            DebugExecutionLimits {
                max_instructions: 2,
                ..DebugExecutionLimits::default()
            },
            "instruction_limit",
        ),
        (
            "program Main; begin while true do begin end end.",
            DebugExecutionLimits {
                timeout: Duration::ZERO,
                ..DebugExecutionLimits::default()
            },
            "timeout",
        ),
    ];
    for (source, limits, code) in cases {
        let mut server = server_with_limits(source, limits);
        let _ = server.handle_line(&request(1, "initialize", json!({})));
        let _ = server.handle_line(&request(2, "launch", json!({"stop_on_entry":false})));
        let records = server.wait();
        assert!(
            records.iter().any(|record| record["body"]["code"] == code),
            "missing {code}: {records:?}"
        );
    }
}

#[test]
fn initialize_reports_the_configured_execution_limits() {
    let limits = DebugExecutionLimits {
        max_instructions: 123,
        timeout: Duration::from_millis(456),
        max_output_bytes: 789,
    };
    let mut server = server_with_limits("program Main; begin end.", limits);

    let records = server.handle_line(&request(1, "initialize", json!({})));
    let reported = &records[0]["body"]["limits"];

    assert_eq!(reported["instructions"], 123);
    assert_eq!(reported["timeout_milliseconds"], 456);
    assert_eq!(reported["captured_output_bytes"], 789);
}

#[test]
fn broken_protocol_writer_is_returned_as_transport_failure() {
    struct BrokenWriter;
    impl std::io::Write for BrokenWriter {
        fn write(&mut self, _buffer: &[u8]) -> std::io::Result<usize> {
            Err(std::io::Error::new(
                std::io::ErrorKind::BrokenPipe,
                "closed",
            ))
        }

        fn flush(&mut self) -> std::io::Result<()> {
            Ok(())
        }
    }

    let script = request(1, "initialize", json!({}));
    let error = serve_script(
        script.as_bytes(),
        BrokenWriter,
        server("program Main; begin end."),
    )
    .expect_err("broken writer must fail transport");
    assert_eq!(error.kind(), std::io::ErrorKind::BrokenPipe);
}
