//! Executable coverage for the repository JSONL protocol contract.

#![allow(
    clippy::expect_used,
    reason = "protocol tests use expect to keep fixture failures local"
)]

use fpas_debug::{
    PreparedDebugTarget,
    jsonl::{JsonlServer, ServerStatus},
};
use serde_json::Value;

const SOURCE: &str = include_str!("../../../tests/debugger/fixtures/debugger_target.fpas");
const POSITIVE_CONTRACT: &str = include_str!("../../../tests/debugger/contracts/positive.jsonl");

fn server() -> JsonlServer {
    let (program, diagnostics) = fpas_parser::parse(SOURCE);
    assert!(diagnostics.is_empty(), "parse diagnostics: {diagnostics:?}");
    let executable = fpas_compiler::compile(&program).expect("compile debugger contract fixture");
    JsonlServer::new(PreparedDebugTarget::new(executable, Vec::new()))
        .expect("create JSONL contract server")
}

#[test]
fn positive_contract_stops_reports_stack_and_completes_fixture() {
    let mut server = server();
    let mut records = Vec::new();

    for line in POSITIVE_CONTRACT.lines() {
        let request: Value = serde_json::from_str(line).expect("valid contract request");
        let command = request["command"].as_str().expect("contract command");
        let response = server.handle_line(line);
        assert_eq!(response[0]["success"], true, "{command}: {response:?}");
        if command == "breakpoint.set" {
            assert_eq!(response[0]["body"]["verified"], true, "{response:?}");
        }
        if command == "stack" {
            assert_eq!(
                response[0]["body"]["frames"][0]["location"]["line"], 32,
                "{response:?}"
            );
        }
        records.extend(response);
        while server.status() == ServerStatus::Running {
            records.extend(server.wait());
        }
    }

    assert_eq!(server.status(), ServerStatus::Terminated, "{records:?}");
    assert!(records.iter().any(|record| {
        record["event"] == "stopped"
            && record["body"]["reason"] == "breakpoint"
            && record["body"]["location"]["line"] == 32
    }));
    let output = records
        .iter()
        .filter(|record| record["event"] == "output")
        .filter_map(|record| record["body"]["text"].as_str())
        .collect::<String>();
    assert_eq!(output, "[3, 4, 24]\n11\n");
}
