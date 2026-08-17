//! JSONL global assignment attached to source and data breakpoints.

#![allow(
    clippy::expect_used,
    reason = "protocol tests keep fixture failures local"
)]

use fpas_debug::{
    PreparedDebugTarget,
    jsonl::{JsonlServer, ServerStatus},
};
use serde_json::{Value, json};

const SOURCE: &str = r#"program BreakpointAssign;

mutable var Flag: integer := 0;

begin
  Flag := 1;
  Flag := 2
end.
"#;

fn server() -> JsonlServer {
    let (program, diagnostics) = fpas_parser::parse(SOURCE);
    assert!(diagnostics.is_empty(), "parse diagnostics: {diagnostics:?}");
    let executable = fpas_compiler::compile(&program).expect("compile assign fixture");
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
        initialized[0]["body"]["capabilities"]["breakpoint_assign"],
        true
    );
    send(server, id, "launch", json!({"stop_on_entry":true}))
}

fn continue_until_stopped(server: &mut JsonlServer, id: &mut u64) -> Vec<Value> {
    let mut records = send(server, id, "continue", json!({}));
    if !records.iter().any(|record| record["event"] == "stopped") {
        records = server.wait();
    }
    records
}

fn line(needle: &str) -> u32 {
    u32::try_from(
        SOURCE
            .lines()
            .position(|line| line.contains(needle))
            .expect("marker")
            + 1,
    )
    .expect("line")
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

fn flag_identity(server: &mut JsonlServer, id: &mut u64) -> Value {
    let globals = globals_reference(server, id);
    send(
        server,
        id,
        "location.describe",
        json!({"variables_reference":globals,"name":"Flag"}),
    )[0]["body"]["identity"]
        .clone()
}

fn evaluate_flag(server: &mut JsonlServer, id: &mut u64) -> String {
    send(server, id, "evaluate", json!({"expression":"Flag"}))[0]["body"]["result"]
        .as_str()
        .expect("Flag")
        .to_string()
}

#[test]
fn jsonl_source_breakpoint_assign_commits_before_the_line() {
    let mut server = server();
    let mut id = 0;
    let launched = launch_stopped(&mut server, &mut id);
    assert!(
        launched.iter().any(|record| record["event"] == "stopped"),
        "{launched:?}"
    );
    let identity = flag_identity(&mut server, &mut id);
    let set = send(
        &mut server,
        &mut id,
        "breakpoint.set",
        json!({
            "source":"<memory>",
            "line":line("Flag := 2"),
            "assign":{"identity":identity,"expression":"99"}
        }),
    );
    assert_eq!(set[0]["success"], true, "{set:?}");
    assert_eq!(set[0]["body"]["verified"], true, "{set:?}");

    let stopped = continue_until_stopped(&mut server, &mut id);
    assert!(
        stopped.iter().any(|record| record["event"] == "stopped"),
        "{stopped:?}"
    );
    assert_eq!(evaluate_flag(&mut server, &mut id), "99");
}

#[test]
fn jsonl_frame_assign_is_rejected_without_creating_the_breakpoint() {
    let mut server = server();
    let mut id = 0;
    let _ = launch_stopped(&mut server, &mut id);
    let rejected = send(
        &mut server,
        &mut id,
        "breakpoint.set",
        json!({
            "source":"<memory>",
            "line":line("Flag := 2"),
            "assign":{
                "identity":{"task_id":0,"function":0,"register":0},
                "expression":"99"
            }
        }),
    );
    assert_eq!(rejected[0]["success"], false, "{rejected:?}");
    assert_eq!(rejected[0]["error"]["code"], "invalid_request");
    assert_eq!(rejected.len(), 1, "{rejected:?}");

    let continued = continue_until_stopped(&mut server, &mut id);
    assert!(
        continued
            .iter()
            .any(|record| record["event"] == "terminated")
            || server.status() == ServerStatus::Terminated,
        "{continued:?}"
    );
}

#[test]
fn jsonl_failed_assign_stops_without_mutating() {
    let mut server = server();
    let mut id = 0;
    let _ = launch_stopped(&mut server, &mut id);
    let identity = flag_identity(&mut server, &mut id);
    let set = send(
        &mut server,
        &mut id,
        "breakpoint.set",
        json!({
            "source":"<memory>",
            "line":line("Flag := 2"),
            "assign":{"identity":identity,"expression":"true"}
        }),
    );
    assert_eq!(set[0]["success"], true, "{set:?}");

    let stopped = continue_until_stopped(&mut server, &mut id);
    assert!(
        stopped
            .iter()
            .any(|record| record["event"] == "protocol_error"),
        "{stopped:?}"
    );
    assert_eq!(evaluate_flag(&mut server, &mut id), "1");
}

#[test]
fn jsonl_logpoint_assign_continues_and_logs_the_new_value() {
    let mut server = server();
    let mut id = 0;
    let _ = launch_stopped(&mut server, &mut id);
    let identity = flag_identity(&mut server, &mut id);
    let set = send(
        &mut server,
        &mut id,
        "breakpoint.set",
        json!({
            "source":"<memory>",
            "line":line("Flag := 2"),
            "assign":{"identity":identity,"expression":"99"},
            "log_message":"Flag={Flag}"
        }),
    );
    assert_eq!(set[0]["success"], true, "{set:?}");

    let mut records = send(&mut server, &mut id, "continue", json!({}));
    while server.status() == ServerStatus::Running {
        records.extend(server.wait());
    }
    let output = records
        .iter()
        .filter(|record| record["event"] == "output")
        .filter_map(|record| record["body"]["text"].as_str())
        .collect::<String>();
    assert!(output.contains("Flag=99"), "{records:?}");
    assert_eq!(server.status(), ServerStatus::Terminated, "{records:?}");
}

#[test]
fn jsonl_data_breakpoint_assign_runs_after_the_watched_store() {
    let mut server = server();
    let mut id = 0;
    let _ = launch_stopped(&mut server, &mut id);
    let identity = flag_identity(&mut server, &mut id);
    let replaced = send(
        &mut server,
        &mut id,
        "data_breakpoints.replace",
        json!({
            "breakpoints":[{
                "identity":identity,
                "access":"write",
                "assign":{"identity":identity,"expression":"0"}
            }]
        }),
    );
    assert_eq!(replaced[0]["success"], true, "{replaced:?}");
    assert_eq!(replaced[0]["body"]["breakpoints"][0]["verified"], true);

    let stopped = continue_until_stopped(&mut server, &mut id);
    let event = stopped
        .iter()
        .find(|record| record["event"] == "stopped")
        .expect("data stop");
    assert_eq!(event["body"]["reason"], "data_breakpoint", "{stopped:?}");
    assert_eq!(evaluate_flag(&mut server, &mut id), "0");
}

#[test]
fn jsonl_function_breakpoints_reject_assign() {
    let mut server = server();
    let mut id = 0;
    let _ = send(&mut server, &mut id, "initialize", json!({"version":2}));
    let rejected = send(
        &mut server,
        &mut id,
        "function_breakpoints.replace",
        json!({
            "breakpoints":[{
                "name":"Main",
                "assign":{"identity":{"index":0},"expression":"1"}
            }]
        }),
    );
    assert_eq!(rejected[0]["success"], false, "{rejected:?}");
    assert_eq!(rejected[0]["error"]["code"], "invalid_request");
}
