//! DAP durable data-location identities through the JSONL core.

#![allow(
    clippy::expect_used,
    reason = "protocol tests keep fixture failures local"
)]

use fpas_debug::{PreparedDebugTarget, dap::DapServer};
use serde_json::{Value, json};

const SOURCE: &str = r#"program DataLocation;

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
    let executable = fpas_compiler::compile(&program).expect("compile data-location fixture");
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

fn stop_at(server: &mut DapServer, seq: &mut u64, needle: &str) {
    let line = SOURCE
        .lines()
        .position(|line| line.contains(needle))
        .expect("marker")
        + 1;
    let _ = send(
        server,
        seq,
        "setBreakpoints",
        json!({"source":{"path":"<memory>"},"breakpoints":[{"line":line}]}),
    );
    let _ = send(server, seq, "continue", json!({"threadId":1}));
    let _ = server.wait();
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

fn locals_reference(server: &mut DapServer, seq: &mut u64) -> u64 {
    let stack = send(server, seq, "stackTrace", json!({"threadId":1}));
    let frame = stack[0]["body"]["stackFrames"][0]["id"]
        .as_u64()
        .expect("frame");
    send(server, seq, "scopes", json!({"frameId":frame}))[0]["body"]["scopes"]
        .as_array()
        .expect("scopes")
        .iter()
        .find(|scope| scope["name"] == "Locals")
        .and_then(|scope| scope["variablesReference"].as_u64())
        .expect("locals")
}

#[test]
fn dap_location_describe_names_globals_across_continue() {
    let mut server = server();
    let mut seq = 0;
    start(&mut server, &mut seq);
    stop_at(&mut server, &mut seq, "Flag := 1;");
    let globals = globals_reference(&mut server, &mut seq);
    let first = send(
        &mut server,
        &mut seq,
        "fpas/locationDescribe",
        json!({"variablesReference":globals,"name":"Flag"}),
    );
    assert_eq!(first[0]["success"], true, "{first:?}");
    assert_eq!(first[0]["body"]["kind"], "global");
    assert_eq!(first[0]["body"]["lifetime"], "executable");
    assert_eq!(first[0]["body"]["identity"]["index"], 0);

    stop_at(&mut server, &mut seq, "Flag := 2");
    let globals = globals_reference(&mut server, &mut seq);
    let second = send(
        &mut server,
        &mut seq,
        "fpas/locationDescribe",
        json!({"variablesReference":globals,"name":"Flag"}),
    );
    assert_eq!(second[0]["body"]["identity"], first[0]["body"]["identity"]);
}

#[test]
fn dap_location_describe_expires_with_the_frame_handle() {
    let mut server = server();
    let mut seq = 0;
    start(&mut server, &mut seq);
    stop_at(&mut server, &mut seq, "Nested := Nested + Flag");
    let locals = locals_reference(&mut server, &mut seq);
    let described = send(
        &mut server,
        &mut seq,
        "fpas/locationDescribe",
        json!({"variablesReference":locals,"name":"Nested"}),
    );
    assert_eq!(described[0]["body"]["kind"], "frame_register");
    assert_eq!(described[0]["body"]["lifetime"], "live_frame");
    assert!(described[0]["body"]["identity"]["function"].is_number());

    stop_at(&mut server, &mut seq, "Flag := 2");
    let rejected = send(
        &mut server,
        &mut seq,
        "fpas/locationDescribe",
        json!({"variablesReference":locals,"name":"Nested"}),
    );
    assert_eq!(rejected[0]["success"], false, "{rejected:?}");
    assert_eq!(rejected.len(), 1, "{rejected:?}");
}
