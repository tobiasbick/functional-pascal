//! JSONL runtime-failure filter contracts and failed-termination semantics.

#![allow(
    clippy::expect_used,
    reason = "protocol fixtures use expect to keep failures local"
)]

use fpas_debug::{
    PreparedDebugTarget,
    jsonl::{JsonlServer, ServerStatus},
};
use serde_json::{Value, json};

const DIVISION_BY_ZERO: &str = r#"program RuntimeFailure;

begin
  var Zero: integer := 0;
  var Value: integer := 1 div Zero
end.
"#;

fn server() -> JsonlServer {
    let (program, diagnostics) = fpas_parser::parse(DIVISION_BY_ZERO);
    assert!(diagnostics.is_empty(), "{diagnostics:?}");
    let executable = fpas_compiler::compile(&program).expect("compile failure fixture");
    JsonlServer::new(PreparedDebugTarget::new(executable, Vec::new())).expect("create JSONL server")
}

fn request(id: u64, command: &str, arguments: Value) -> String {
    json!({"type":"request","id":id,"command":command,"arguments":arguments}).to_string()
}

fn run(server: &mut JsonlServer) -> Vec<Value> {
    let _ = server.handle_line(&request(10, "launch", json!({"stop_on_entry":false})));
    let mut records = Vec::new();
    while server.status() == ServerStatus::Running {
        records.extend(server.wait());
    }
    records
}

#[test]
fn default_and_exact_matching_filters_keep_runtime_failures_inspectable() {
    for filters in [None, Some(json!(["F4001"]))] {
        let mut server = server();
        let initialized = server.handle_line(&request(1, "initialize", json!({})));
        assert_eq!(
            initialized[0]["body"]["capabilities"]["runtime_failure_filters"],
            true
        );
        assert_eq!(
            initialized[0]["body"]["limits"]["runtime_failure_filters"],
            64
        );
        if let Some(filters) = filters {
            let configured = server.handle_line(&request(
                2,
                "runtime_failures.replace",
                json!({"filters": filters}),
            ));
            assert_eq!(configured[0]["success"], true, "{configured:?}");
        }

        let records = run(&mut server);
        assert!(
            records.iter().any(|record| {
                record["event"] == "runtime_error" && record["body"]["code"] == "F4001"
            }),
            "{records:?}"
        );
        assert!(
            records.iter().any(|record| {
                record["event"] == "stopped" && record["body"]["reason"] == "runtime_error"
            }),
            "{records:?}"
        );
        assert_eq!(server.status(), ServerStatus::Stopped);
    }
}

#[test]
fn nonmatching_filter_reports_diagnostic_then_failed_termination_without_stop() {
    let mut server = server();
    let _ = server.handle_line(&request(1, "initialize", json!({})));
    let configured = server.handle_line(&request(
        2,
        "runtime_failures.replace",
        json!({"filters":["F4010"]}),
    ));
    assert_eq!(configured[0]["success"], true);

    let records = run(&mut server);
    let runtime_error = records
        .iter()
        .position(|record| record["event"] == "runtime_error")
        .expect("runtime diagnostic event");
    let terminated = records
        .iter()
        .position(|record| record["event"] == "terminated")
        .expect("failed termination event");
    assert!(runtime_error < terminated, "{records:?}");
    assert!(!records.iter().any(|record| record["event"] == "stopped"));
    assert_eq!(records[terminated]["body"]["reason"], "runtime_error");
    assert_eq!(records[terminated]["body"]["exit_code"], 1);
    assert_eq!(records[terminated]["body"]["diagnostic_code"], "F4001");
    assert_eq!(server.status(), ServerStatus::Terminated);
}

#[test]
fn invalid_replacement_is_atomic() {
    let excessive = Value::Array(
        (0..65)
            .map(|_| Value::String("F4001".to_string()))
            .collect(),
    );
    for invalid in [
        json!(["F4016"]),
        json!(["F4001", "F4001"]),
        json!(["all", "F4001"]),
        excessive,
    ] {
        let mut server = server();
        let _ = server.handle_line(&request(1, "initialize", json!({})));
        let rejected = server.handle_line(&request(
            2,
            "runtime_failures.replace",
            json!({"filters": invalid}),
        ));
        assert_eq!(rejected[0]["success"], false);
        assert_eq!(rejected[0]["error"]["code"], "invalid_request");

        let records = run(&mut server);
        assert!(
            records.iter().any(|record| record["event"] == "stopped"),
            "{records:?}"
        );
    }
}
