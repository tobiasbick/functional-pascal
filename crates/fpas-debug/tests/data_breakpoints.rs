//! JSONL global data breakpoints through durable location identities.

#![allow(
    clippy::expect_used,
    reason = "protocol tests keep fixture failures local"
)]

use fpas_debug::{
    PreparedDebugTarget,
    jsonl::{JsonlServer, ServerStatus},
};
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

fn server() -> JsonlServer {
    let (program, diagnostics) = fpas_parser::parse(SOURCE);
    assert!(diagnostics.is_empty(), "parse diagnostics: {diagnostics:?}");
    let executable = fpas_compiler::compile(&program).expect("compile data-breakpoint fixture");
    JsonlServer::new(PreparedDebugTarget::new(executable, Vec::new())).expect("JSONL server")
}

fn request(id: u64, command: &str, arguments: Value) -> String {
    json!({"type":"request","id":id,"command":command,"arguments":arguments}).to_string()
}

fn send(server: &mut JsonlServer, id: &mut u64, command: &str, arguments: Value) -> Vec<Value> {
    *id += 1;
    server.handle_line(&request(*id, command, arguments))
}

fn launch_stopped(server: &mut JsonlServer, id: &mut u64) -> Vec<Value> {
    let initialized = send(server, id, "initialize", json!({"version":2}));
    assert_eq!(
        initialized[0]["body"]["capabilities"]["data_breakpoints"],
        true
    );
    assert_eq!(
        initialized[0]["body"]["capabilities"]["data_breakpoint_access"],
        json!(["write", "change"])
    );
    send(server, id, "launch", json!({"stop_on_entry":true}))
}

fn globals_reference(server: &mut JsonlServer, id: &mut u64) -> u64 {
    let frame = send(server, id, "stack", json!({}))[0]["body"]["frames"][0]["frame_id"]
        .as_u64()
        .expect("frame");
    send(server, id, "scopes", json!({"frame_id":frame}))[0]["body"]["scopes"]
        .as_array()
        .expect("scopes")
        .iter()
        .find(|scope| scope["name"] == "Globals")
        .and_then(|scope| scope["variables_reference"].as_u64())
        .expect("globals")
}

fn locals_reference(server: &mut JsonlServer, id: &mut u64) -> u64 {
    let frame = send(server, id, "stack", json!({}))[0]["body"]["frames"][0]["frame_id"]
        .as_u64()
        .expect("frame");
    send(server, id, "scopes", json!({"frame_id":frame}))[0]["body"]["scopes"]
        .as_array()
        .expect("scopes")
        .iter()
        .find(|scope| scope["name"] == "Locals")
        .and_then(|scope| scope["variables_reference"].as_u64())
        .expect("locals")
}

#[test]
fn jsonl_data_breakpoint_set_stays_rejected() {
    let mut server = server();
    let mut id = 0;
    let _ = launch_stopped(&mut server, &mut id);
    let rejected = send(&mut server, &mut id, "data_breakpoint.set", json!({}));
    assert_eq!(rejected[0]["success"], false, "{rejected:?}");
    assert_eq!(rejected[0]["error"]["code"], "unsupported_capability");
    assert_eq!(rejected.len(), 1, "{rejected:?}");
    assert_eq!(server.status(), ServerStatus::Stopped);
}

#[test]
fn jsonl_data_breakpoints_watch_globals_across_continue() {
    let mut server = server();
    let mut id = 0;
    let launched = launch_stopped(&mut server, &mut id);
    assert!(
        launched.iter().any(|record| record["event"] == "stopped"),
        "{launched:?}"
    );

    let globals = globals_reference(&mut server, &mut id);
    let described = send(
        &mut server,
        &mut id,
        "location.describe",
        json!({"variables_reference":globals,"name":"Flag"}),
    );
    let identity = described[0]["body"]["identity"].clone();
    assert_eq!(identity["index"], 0, "{described:?}");

    let replaced = send(
        &mut server,
        &mut id,
        "data_breakpoints.replace",
        json!({"breakpoints":[{"identity":identity,"access":"write"}]}),
    );
    assert_eq!(replaced[0]["success"], true, "{replaced:?}");
    assert_eq!(replaced[0]["body"]["breakpoints"][0]["verified"], true);
    let breakpoint_id = replaced[0]["body"]["breakpoints"][0]["breakpoint_id"]
        .as_u64()
        .expect("id");

    let _ = send(&mut server, &mut id, "continue", json!({}));
    let stopped = server.wait();
    let event = stopped
        .iter()
        .find(|record| record["event"] == "stopped")
        .expect("data stop");
    assert_eq!(event["body"]["reason"], "data_breakpoint", "{stopped:?}");
    assert_eq!(event["body"]["breakpoint_ids"], json!([breakpoint_id]));
    assert_eq!(server.status(), ServerStatus::Stopped);

    let globals = globals_reference(&mut server, &mut id);
    let again = send(
        &mut server,
        &mut id,
        "location.describe",
        json!({"variables_reference":globals,"name":"Flag"}),
    );
    assert_eq!(again[0]["body"]["identity"]["index"], 0);
}

#[test]
fn jsonl_data_breakpoint_reject_does_not_resume() {
    let mut server = server();
    let mut id = 0;
    let _ = launch_stopped(&mut server, &mut id);
    let frame =
        send(&mut server, &mut id, "stack", json!({}))[0]["body"]["frames"][0]["frame_id"].clone();

    let missing = send(&mut server, &mut id, "data_breakpoints.replace", json!({}));
    assert_eq!(
        missing[0]["error"]["code"], "invalid_request",
        "{missing:?}"
    );
    assert_eq!(missing.len(), 1, "{missing:?}");

    let invalid = send(
        &mut server,
        &mut id,
        "data_breakpoints.replace",
        json!({"breakpoints":[{"identity":{},"access":"write"}]}),
    );
    assert_eq!(
        invalid[0]["error"]["code"], "invalid_request",
        "{invalid:?}"
    );
    assert_eq!(server.status(), ServerStatus::Stopped);
    let same_stack = send(&mut server, &mut id, "stack", json!({}));
    assert_eq!(same_stack[0]["body"]["frames"][0]["frame_id"], frame);
}

#[test]
fn jsonl_frame_data_breakpoint_stays_unverified() {
    let mut server = server();
    let mut id = 0;
    let _ = launch_stopped(&mut server, &mut id);
    let line = SOURCE
        .lines()
        .position(|line| line.contains("Nested := Nested + Flag"))
        .expect("marker")
        + 1;
    let _ = send(
        &mut server,
        &mut id,
        "breakpoint.set",
        json!({"source":"<memory>","line":line}),
    );
    let _ = send(&mut server, &mut id, "continue", json!({}));
    let _ = server.wait();

    let locals = locals_reference(&mut server, &mut id);
    let described = send(
        &mut server,
        &mut id,
        "location.describe",
        json!({"variables_reference":locals,"name":"Nested"}),
    );
    let replaced = send(
        &mut server,
        &mut id,
        "data_breakpoints.replace",
        json!({"breakpoints":[{"identity":described[0]["body"]["identity"],"access":"write"}]}),
    );
    assert_eq!(replaced[0]["body"]["breakpoints"][0]["verified"], false);
    assert_eq!(server.status(), ServerStatus::Stopped);
}
