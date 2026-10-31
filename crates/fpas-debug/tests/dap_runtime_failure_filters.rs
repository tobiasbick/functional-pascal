//! DAP exception-breakpoint filtering and event-order contracts.

#![allow(
    clippy::expect_used,
    reason = "DAP fixtures use expect to keep transcript failures local"
)]

use fpas_debug::{PreparedDebugTarget, dap::DapServer};
use serde_json::{Value, json};

fn server() -> DapServer {
    let source =
        "program Main; begin const Zero: integer := 0; const Value: integer := 1 div Zero; end.";
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
    assert!(advertised.iter().any(|filter| filter["filter"] == "FP5001"));
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
    let (server, messages) = configure_and_run(json!(["FP5001"]));
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
    let (server, messages) = configure_and_run(json!(["FP5010"]));
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

#[test]
fn supplemental_cleanup_diagnostic_does_not_stop_successful_completion() {
    let source = r#"program CleanupDiagnostic;
type Resource = record
  release procedure Close(Self: Resource);
  begin
    panic('cleanup failed');
  end procedure;
end record;
function Failure(): result of (integer, string);
begin
  return Error('original');
end function;
function Work(): result of (integer, string);
begin
  using const Item := Resource();
  return Ok(try Failure());
end function;
begin
  if not (Work() is Error('original')) then panic('result lost'); end if;
end.
"#;
    let (program, diagnostics) = fpas_parser::parse(source);
    assert!(diagnostics.is_empty(), "{diagnostics:?}");
    let executable = fpas_compiler::compile(&program).expect("compile cleanup fixture");
    let mut server = DapServer::new(PreparedDebugTarget::new(executable, Vec::new()))
        .expect("create DAP server");
    let _ = server.handle(request(1, "initialize", json!({})));
    let _ = server.handle(request(2, "launch", json!({"stopOnEntry":false})));
    let mut messages = server.handle(request(3, "configurationDone", json!({})));
    while server.is_running() {
        messages.extend(server.wait());
    }
    assert!(
        messages.iter().any(|message| {
            message["event"] == "output"
                && message["body"]["category"] == "stderr"
                && message["body"]["output"]
                    .as_str()
                    .is_some_and(|text| text.contains("FP5010") && text.contains("cleanup failed"))
        }),
        "{messages:?}"
    );
    assert!(
        messages
            .iter()
            .any(|message| { message["event"] == "exited" && message["body"]["exitCode"] == 0 }),
        "{messages:?}"
    );
    assert!(!messages.iter().any(|message| message["event"] == "stopped"));
}
