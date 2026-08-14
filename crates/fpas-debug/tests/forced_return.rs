//! JSONL forced-return coverage.

#![allow(
    clippy::expect_used,
    clippy::panic,
    reason = "protocol tests keep fixture failures local"
)]

use fpas_debug::{PreparedDebugTarget, jsonl::JsonlServer};
use serde_json::{Value, json};

const SOURCE: &str = include_str!("../../../tests/debugger/fixtures/forced_return.fpas");

fn server() -> JsonlServer {
    let (program, diagnostics) = fpas_parser::parse(SOURCE);
    assert!(diagnostics.is_empty(), "parse diagnostics: {diagnostics:?}");
    let executable = fpas_compiler::compile(&program).expect("compile forced-return fixture");
    JsonlServer::new(PreparedDebugTarget::new(executable, Vec::new())).expect("JSONL server")
}

fn request(id: u64, command: &str, arguments: Value) -> String {
    json!({"type":"request","id":id,"command":command,"arguments":arguments}).to_string()
}

fn send(server: &mut JsonlServer, id: &mut u64, command: &str, arguments: Value) -> Vec<Value> {
    *id += 1;
    server.handle_line(&request(*id, command, arguments))
}

fn stack_frames(server: &mut JsonlServer, id: &mut u64) -> Vec<Value> {
    send(server, id, "stack", json!({}))[0]["body"]["frames"]
        .as_array()
        .expect("frames")
        .clone()
}

fn stop_in_function(server: &mut JsonlServer, id: &mut u64, name: &str) -> u64 {
    for _ in 0..64 {
        let frames = stack_frames(server, id);
        if frames
            .first()
            .is_some_and(|frame| frame["name"] == name && frame["depth"] == 0)
        {
            return frames[0]["frame_id"].as_u64().expect("frame ID");
        }
        let _ = send(server, id, "step_into", json!({}));
        let _ = server.wait();
    }
    panic!("{name} never became the active callee")
}

fn initialize(server: &mut JsonlServer) -> u64 {
    let mut id = 0;
    let initialized = send(server, &mut id, "initialize", json!({"version": 2}));
    assert_eq!(initialized[0]["body"]["capabilities"]["frame_return"], true);
    let _ = send(server, &mut id, "launch", json!({"stop_on_entry": true}));
    id
}

#[test]
fn jsonl_frame_return_completes_a_function_and_continues_from_the_caller() {
    let mut server = server();
    let mut id = initialize(&mut server);
    let callee = stop_in_function(&mut server, &mut id, "compute");
    let mismatch = send(
        &mut server,
        &mut id,
        "frame.return",
        json!({"frame_id": callee, "expression": "'nope'"}),
    );
    assert_eq!(mismatch[0]["error"]["code"], "frame_return_type");
    assert!(
        mismatch[0]["error"]["help"]
            .as_str()
            .is_some_and(|help| !help.is_empty()),
        "{mismatch:?}"
    );

    let returned = send(
        &mut server,
        &mut id,
        "frame.return",
        json!({"frame_id": callee, "expression": "PlusOne(41)"}),
    );
    assert_eq!(returned[0]["success"], true, "{returned:?}");
    assert_eq!(returned[0]["body"]["result"], "42");
    assert_eq!(returned[0]["body"]["type_name"], "integer");
    assert_eq!(returned[0]["body"]["unwound_frames"], 1);
    assert_eq!(returned[0]["body"]["frame"]["name"], "forcedreturn");
    assert_eq!(returned[0]["body"]["frame"]["depth"], 0);

    let caller = stack_frames(&mut server, &mut id)[0]["frame_id"]
        .as_u64()
        .expect("caller");
    let answer = send(
        &mut server,
        &mut id,
        "evaluate",
        json!({"frame_id": caller, "expression": "Answer"}),
    );
    assert_eq!(answer[0]["body"]["result"], "42", "{answer:?}");

    let _ = send(&mut server, &mut id, "continue", json!({}));
    let output = server
        .wait()
        .iter()
        .filter(|record| record["event"] == "output")
        .filter_map(|record| record["body"]["text"].as_str())
        .collect::<String>();
    assert_eq!(output, "skip me\n42\n", "{output:?}");
}

#[test]
fn jsonl_frame_return_completes_a_procedure_and_rejects_convention_errors() {
    let mut server = server();
    let mut id = initialize(&mut server);
    let function = stop_in_function(&mut server, &mut id, "compute");
    let missing = send(
        &mut server,
        &mut id,
        "frame.return",
        json!({"frame_id": function}),
    );
    assert_eq!(missing[0]["error"]["code"], "frame_return_value_required");
    let _ = send(
        &mut server,
        &mut id,
        "frame.return",
        json!({"frame_id": function, "expression": "42"}),
    );

    let procedure = stop_in_function(&mut server, &mut id, "announce");
    let unexpected = send(
        &mut server,
        &mut id,
        "frame.return",
        json!({"frame_id": procedure, "expression": "'nope'"}),
    );
    assert_eq!(
        unexpected[0]["error"]["code"],
        "frame_return_value_unexpected"
    );
    let returned = send(
        &mut server,
        &mut id,
        "frame.return",
        json!({"frame_id": procedure}),
    );
    assert_eq!(returned[0]["success"], true, "{returned:?}");
    assert_eq!(returned[0]["body"]["result"], "()");
    assert_eq!(returned[0]["body"]["unwound_frames"], 1);

    let _ = send(&mut server, &mut id, "continue", json!({}));
    let output = server
        .wait()
        .iter()
        .filter(|record| record["event"] == "output")
        .filter_map(|record| record["body"]["text"].as_str())
        .collect::<String>();
    assert_eq!(output, "42\n", "{output:?}");
}

#[test]
fn jsonl_frame_return_stays_bound_to_the_selected_stop_task() {
    const TASK_SOURCE: &str = r#"program TaskForcedReturn;

uses Std.Console, Std.Task, Std.Time;

function Work(): integer;
begin
  Sleep(30000);
  return 1
end;

function Compute(Value: integer): integer;
begin
  return Value + 1
end;

begin
  var Pending: task := go Work();
  var Answer: integer := Compute(41);
  WriteLn(Answer)
end.
"#;
    let (program, diagnostics) = fpas_parser::parse(TASK_SOURCE);
    assert!(diagnostics.is_empty(), "{diagnostics:?}");
    let executable = fpas_compiler::compile(&program).expect("compile task fixture");
    let mut server =
        JsonlServer::new(PreparedDebugTarget::new(executable, Vec::new())).expect("server");
    let mut id = initialize(&mut server);
    let callee = stop_in_function(&mut server, &mut id, "compute");
    id += 1;
    let peer = server.handle_line(&request(id, "stack", json!({"task_id": 1})));
    let peer_frame = peer[0]["body"]["frames"][0]["frame_id"]
        .as_u64()
        .expect("peer frame");
    let rejected = send(
        &mut server,
        &mut id,
        "frame.return",
        json!({"frame_id": peer_frame, "expression": "9"}),
    );
    assert_eq!(
        rejected[0]["error"]["code"], "frame_return_unsupported",
        "{rejected:?}"
    );
    let returned = send(
        &mut server,
        &mut id,
        "frame.return",
        json!({"frame_id": callee, "expression": "9"}),
    );
    assert_eq!(returned[0]["success"], true, "{returned:?}");
    assert_eq!(returned[0]["body"]["result"], "9");
    assert_eq!(returned[0]["body"]["unwound_frames"], 1);
}

fn frame_id_at_depth(frames: &[Value], depth: u64) -> u64 {
    frames
        .iter()
        .find(|frame| frame["depth"] == depth)
        .and_then(|frame| frame["frame_id"].as_u64())
        .unwrap_or_else(|| panic!("stack should include depth {depth}"))
}

#[test]
fn jsonl_frame_return_completes_a_selected_older_frame() {
    let mut server = server();
    let mut id = initialize(&mut server);
    let _leaf = stop_in_function(&mut server, &mut id, "leaf");
    let frames = stack_frames(&mut server, &mut id);
    let branch = frame_id_at_depth(&frames, 1);
    let unknown = send(
        &mut server,
        &mut id,
        "frame.return",
        json!({"frame_id": 1, "expression": "1"}),
    );
    assert_eq!(unknown[0]["error"]["code"], "unknown_frame", "{unknown:?}");

    let returned = send(
        &mut server,
        &mut id,
        "frame.return",
        json!({"frame_id": branch, "expression": "Local"}),
    );
    assert_eq!(returned[0]["success"], true, "{returned:?}");
    assert_eq!(returned[0]["body"]["result"], "11");
    assert_eq!(returned[0]["body"]["unwound_frames"], 2);
    assert_eq!(returned[0]["body"]["frame"]["name"], "forcedreturn");
    assert_eq!(returned[0]["body"]["frame"]["depth"], 0);

    let caller = stack_frames(&mut server, &mut id)[0]["frame_id"]
        .as_u64()
        .expect("caller");
    let nested = send(
        &mut server,
        &mut id,
        "evaluate",
        json!({"frame_id": caller, "expression": "Nested"}),
    );
    assert_eq!(nested[0]["body"]["result"], "11", "{nested:?}");

    let _ = send(&mut server, &mut id, "continue", json!({}));
    let output = server
        .wait()
        .iter()
        .filter(|record| record["event"] == "output")
        .filter_map(|record| record["body"]["text"].as_str())
        .collect::<String>();
    assert_eq!(output, "11\nskip me\n42\n", "{output:?}");
}
