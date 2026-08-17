//! DAP global data breakpoints through the JSONL core.

#![allow(
    clippy::expect_used,
    reason = "protocol tests keep fixture failures local"
)]

use fpas_debug::{PreparedDebugTarget, dap::DapServer};
use serde_json::{Value, json};

const SOURCE: &str = r#"program DataBreakpoints;

mutable var Flag: integer := 0;

procedure Inner();
begin
  mutable var Nested: integer := 1;
  Nested := Nested + Flag
end;

begin
  Flag := 1;
  Inner();
  Flag := 2
end.
"#;

fn server() -> DapServer {
    let (program, diagnostics) = fpas_parser::parse(SOURCE);
    assert!(diagnostics.is_empty(), "parse diagnostics: {diagnostics:?}");
    let executable = fpas_compiler::compile(&program).expect("compile data-breakpoint fixture");
    DapServer::new(PreparedDebugTarget::new(executable, Vec::new())).expect("DAP server")
}

fn send(server: &mut DapServer, seq: &mut u64, command: &str, arguments: Value) -> Vec<Value> {
    *seq += 1;
    server.handle(json!({
        "seq":*seq,"type":"request","command":command,"arguments":arguments
    }))
}

fn start(server: &mut DapServer, seq: &mut u64) {
    let initialized = send(server, seq, "initialize", json!({}));
    assert_eq!(initialized[0]["body"]["supportsDataBreakpoints"], true);
    let _ = send(server, seq, "launch", json!({"stopOnEntry":true}));
    let configured = send(server, seq, "configurationDone", json!({}));
    assert!(
        configured
            .iter()
            .any(|message| message["event"] == "stopped"),
        "{configured:?}"
    );
}

fn globals_reference(server: &mut DapServer, seq: &mut u64) -> u64 {
    let stack = send(server, seq, "stackTrace", json!({"threadId":1}));
    let frame = stack[0]["body"]["stackFrames"][0]["id"]
        .as_u64()
        .expect("frame");
    send(server, seq, "scopes", json!({"frameId":frame}))[0]["body"]["scopes"]
        .as_array()
        .expect("scopes")
        .iter()
        .find(|scope| scope["name"] == "Globals")
        .and_then(|scope| scope["variablesReference"].as_u64())
        .expect("globals")
}

#[test]
fn dap_data_breakpoints_watch_globals_across_continue() {
    let mut server = server();
    let mut seq = 0;
    start(&mut server, &mut seq);

    let globals = globals_reference(&mut server, &mut seq);
    let info = send(
        &mut server,
        &mut seq,
        "dataBreakpointInfo",
        json!({"variablesReference":globals,"name":"Flag"}),
    );
    assert_eq!(info[0]["success"], true, "{info:?}");
    assert_eq!(info[0]["body"]["dataId"], "g:0");
    assert_eq!(info[0]["body"]["accessTypes"], json!(["write"]));
    assert_eq!(info[0]["body"]["canPersist"], false);

    let set = send(
        &mut server,
        &mut seq,
        "setDataBreakpoints",
        json!({"breakpoints":[{"dataId":"g:0","accessType":"write"}]}),
    );
    assert_eq!(set[0]["success"], true, "{set:?}");
    assert_eq!(set[0]["body"]["breakpoints"][0]["verified"], true);
    let breakpoint_id = set[0]["body"]["breakpoints"][0]["id"].as_u64().expect("id");

    let continued = send(&mut server, &mut seq, "continue", json!({"threadId":1}));
    let mut stopped = continued;
    if !stopped.iter().any(|message| message["event"] == "stopped") {
        stopped = server.wait();
    }
    let event = stopped
        .iter()
        .find(|message| message["event"] == "stopped")
        .expect("data stop");
    assert_eq!(event["body"]["reason"], "data breakpoint", "{stopped:?}");
    let _ = breakpoint_id;

    let globals = globals_reference(&mut server, &mut seq);
    let again = send(
        &mut server,
        &mut seq,
        "dataBreakpointInfo",
        json!({"variablesReference":globals,"name":"Flag"}),
    );
    assert_eq!(again[0]["body"]["dataId"], "g:0");
}

#[test]
fn dap_data_breakpoint_reject_does_not_resume() {
    let mut server = server();
    let mut seq = 0;
    start(&mut server, &mut seq);
    let stack = send(&mut server, &mut seq, "stackTrace", json!({"threadId":1}));
    let frame = stack[0]["body"]["stackFrames"][0]["id"]
        .as_u64()
        .expect("entry frame");

    let rejected = send(
        &mut server,
        &mut seq,
        "setDataBreakpoints",
        json!({"breakpoints":[{"dataId":"not-a-global","accessType":"write"}]}),
    );
    assert_eq!(rejected[0]["success"], false, "{rejected:?}");
    assert_eq!(rejected.len(), 1, "{rejected:?}");

    let same_stack = send(&mut server, &mut seq, "stackTrace", json!({"threadId":1}));
    assert_eq!(same_stack[0]["body"]["stackFrames"][0]["id"], frame);
}

#[test]
fn dap_frame_variable_is_not_watchable() {
    let mut server = server();
    let mut seq = 0;
    start(&mut server, &mut seq);
    let line = SOURCE
        .lines()
        .position(|line| line.contains("Nested := Nested + Flag"))
        .expect("marker")
        + 1;
    let _ = send(
        &mut server,
        &mut seq,
        "setBreakpoints",
        json!({"source":{"path":"<memory>"},"breakpoints":[{"line":line}]}),
    );
    let _ = send(&mut server, &mut seq, "continue", json!({"threadId":1}));
    let _ = server.wait();

    let stack = send(&mut server, &mut seq, "stackTrace", json!({"threadId":1}));
    let frame = stack[0]["body"]["stackFrames"][0]["id"]
        .as_u64()
        .expect("frame");
    let locals =
        send(&mut server, &mut seq, "scopes", json!({"frameId":frame}))[0]["body"]["scopes"]
            .as_array()
            .expect("scopes")
            .iter()
            .find(|scope| scope["name"] == "Locals")
            .and_then(|scope| scope["variablesReference"].as_u64())
            .expect("locals");
    let info = send(
        &mut server,
        &mut seq,
        "dataBreakpointInfo",
        json!({"variablesReference":locals,"name":"Nested"}),
    );
    assert_eq!(info[0]["success"], true, "{info:?}");
    assert_eq!(info[0]["body"]["dataId"], Value::Null);
}
