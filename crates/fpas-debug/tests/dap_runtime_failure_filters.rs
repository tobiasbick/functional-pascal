//! DAP exception-breakpoint filtering and event-order contracts.

#![allow(
    clippy::expect_used,
    reason = "DAP fixtures use expect to keep transcript failures local"
)]

use fpas_debug::{PreparedDebugTarget, dap::DapServer};
use serde_json::{Value, json};

fn server() -> DapServer {
    let source =
        "program Main; begin var Zero: integer := 0; var Value: integer := 1 div Zero end.";
    let (program, diagnostics) = fpas_parser::parse(source);
    assert!(diagnostics.is_empty(), "{diagnostics:?}");
    let executable = fpas_compiler::compile(&program).expect("compile failure fixture");
    DapServer::new(PreparedDebugTarget::new(executable, Vec::new())).expect("create DAP server")
}

fn request(seq: u64, command: &str, arguments: Value) -> Value {
    json!({"seq":seq,"type":"request","command":command,"arguments":arguments})
}

fn configure_and_run(filters: Value) -> (DapServer, Vec<Value>) {
    let mut server = server();
    let initialized = server.handle(request(1, "initialize", json!({})));
    let advertised = initialized[0]["body"]["exceptionBreakpointFilters"]
        .as_array()
        .expect("advertised filters");
    assert!(
        advertised
            .iter()
            .any(|filter| filter["filter"] == "all" && filter["default"] == true)
    );
    assert!(advertised.iter().any(|filter| filter["filter"] == "F4001"));
    let _ = server.handle(request(2, "launch", json!({"stopOnEntry":false})));
    let configured = server.handle(request(
        3,
        "setExceptionBreakpoints",
        json!({"filters": filters}),
    ));
    assert_eq!(configured[0]["success"], true, "{configured:?}");
    let mut messages = server.handle(request(4, "configurationDone", json!({})));
    while server.is_running() {
        messages.extend(server.wait());
    }
    (server, messages)
}

#[test]
fn matching_exception_filter_stops_with_standard_exception_reason() {
    let (server, messages) = configure_and_run(json!(["F4001"]));
    assert!(!server.is_terminated());
    assert!(
        messages.iter().any(|message| {
            message["event"] == "output" && message["body"]["category"] == "stderr"
        }),
        "{messages:?}"
    );
    assert!(
        messages.iter().any(|message| {
            message["event"] == "stopped" && message["body"]["reason"] == "exception"
        }),
        "{messages:?}"
    );
}

#[test]
fn nonmatching_exception_filter_exits_nonzero_without_stopped_event() {
    let (server, messages) = configure_and_run(json!(["F4010"]));
    assert!(server.is_terminated());
    assert!(!messages.iter().any(|message| message["event"] == "stopped"));
    let output = messages
        .iter()
        .position(|message| message["event"] == "output")
        .expect("diagnostic output");
    let exited = messages
        .iter()
        .position(|message| message["event"] == "exited" && message["body"]["exitCode"] == 1)
        .expect("failed exit");
    let terminated = messages
        .iter()
        .position(|message| message["event"] == "terminated")
        .expect("termination");
    assert!(output < exited && exited < terminated, "{messages:?}");
}
