//! JSONL durable data-location identities and capture-cell destination bound.

#![allow(
    clippy::expect_used,
    reason = "protocol tests keep fixture failures local"
)]

use fpas_debug::{
    PreparedDebugTarget,
    jsonl::{JsonlServer, ServerStatus},
};
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

fn server() -> JsonlServer {
    let (program, diagnostics) = fpas_parser::parse(SOURCE);
    assert!(diagnostics.is_empty(), "parse diagnostics: {diagnostics:?}");
    let executable = fpas_compiler::compile(&program).expect("compile data-location fixture");
    JsonlServer::new(PreparedDebugTarget::new(executable, Vec::new())).expect("JSONL server")
}

fn request(id: u64, command: &str, arguments: Value) -> String {
    json!({"type":"request","id":id,"command":command,"arguments":arguments}).to_string()
}

fn send(server: &mut JsonlServer, id: &mut u64, command: &str, arguments: Value) -> Vec<Value> {
    *id += 1;
    server.handle_line(&request(*id, command, arguments))
}

fn launch_stopped(server: &mut JsonlServer, id: &mut u64) {
    let _ = send(server, id, "initialize", json!({"version":2}));
    let launched = send(server, id, "launch", json!({"stop_on_entry":true}));
    assert!(
        launched.iter().any(|record| record["event"] == "stopped"),
        "{launched:?}"
    );
}

fn stop_at(server: &mut JsonlServer, id: &mut u64, needle: &str) {
    let line = SOURCE
        .lines()
        .position(|line| line.contains(needle))
        .expect("marker")
        + 1;
    let breakpoint = send(
        server,
        id,
        "breakpoint.set",
        json!({"source":"<memory>","line":line}),
    );
    assert_eq!(breakpoint[0]["body"]["verified"], true, "{breakpoint:?}");
    let _ = send(server, id, "continue", json!({}));
    let _ = server.wait();
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
fn jsonl_location_describe_names_globals_across_continue() {
    let mut server = server();
    let mut id = 0;
    let initialized = send(&mut server, &mut id, "initialize", json!({"version":2}));
    assert_eq!(
        initialized[0]["body"]["capabilities"]["location_describe"],
        true
    );
    assert_eq!(
        initialized[0]["body"]["capabilities"]["data_breakpoints"],
        true
    );
    let launched = send(
        &mut server,
        &mut id,
        "launch",
        json!({"stop_on_entry":true}),
    );
    assert!(
        launched.iter().any(|record| record["event"] == "stopped"),
        "{launched:?}"
    );

    stop_at(&mut server, &mut id, "Flag := 1;");
    let globals = globals_reference(&mut server, &mut id);
    let first = send(
        &mut server,
        &mut id,
        "location.describe",
        json!({"variables_reference":globals,"name":"Flag"}),
    );
    assert_eq!(first[0]["success"], true, "{first:?}");
    assert_eq!(first[0]["body"]["kind"], "global");
    assert_eq!(first[0]["body"]["lifetime"], "executable");
    assert_eq!(first[0]["body"]["identity"]["index"], 0);
    assert_eq!(server.status(), ServerStatus::Stopped);

    stop_at(&mut server, &mut id, "Flag := 2");
    let globals = globals_reference(&mut server, &mut id);
    let second = send(
        &mut server,
        &mut id,
        "location.describe",
        json!({"variables_reference":globals,"name":"Flag"}),
    );
    assert_eq!(second[0]["body"]["identity"], first[0]["body"]["identity"]);
}

#[test]
fn jsonl_location_describe_expires_with_the_frame_handle() {
    let mut server = server();
    let mut id = 0;
    launch_stopped(&mut server, &mut id);
    stop_at(&mut server, &mut id, "Nested := Nested + Flag");
    let locals = locals_reference(&mut server, &mut id);
    let described = send(
        &mut server,
        &mut id,
        "location.describe",
        json!({"variables_reference":locals,"name":"Nested"}),
    );
    assert_eq!(described[0]["body"]["kind"], "frame_register");
    assert_eq!(described[0]["body"]["lifetime"], "live_frame");
    assert!(described[0]["body"]["identity"]["function"].is_number());

    stop_at(&mut server, &mut id, "Flag := 2");
    let rejected = send(
        &mut server,
        &mut id,
        "location.describe",
        json!({"variables_reference":locals,"name":"Nested"}),
    );
    assert_eq!(rejected[0]["success"], false, "{rejected:?}");
    assert_eq!(rejected[0]["error"]["code"], "variable_target_expired");
    assert_eq!(rejected.len(), 1, "{rejected:?}");
    assert_eq!(server.status(), ServerStatus::Stopped);
}
