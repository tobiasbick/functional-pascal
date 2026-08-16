//! JSONL function-breakpoint identity and atomic policy contracts.

#![allow(
    clippy::expect_used,
    reason = "protocol fixtures use expect to keep failures local"
)]

use fpas_debug::{
    PreparedDebugTarget,
    jsonl::{JsonlServer, ServerStatus},
};
use serde_json::{Value, json};

fn make_server() -> JsonlServer {
    let source = r#"program Main;

function Helper(Value: integer): integer;
begin
  return Value + 1
end;

begin
  var First: integer := Helper(1);
  var Second: integer := Helper(First)
end.
"#;
    let (program, diagnostics) = fpas_parser::parse(source);
    assert!(diagnostics.is_empty(), "{diagnostics:?}");
    let executable = fpas_compiler::compile(&program).expect("compile function fixture");
    JsonlServer::new(PreparedDebugTarget::new(executable, Vec::new())).expect("create JSONL server")
}

fn request(id: u64, command: &str, arguments: Value) -> String {
    json!({"type":"request","id":id,"command":command,"arguments":arguments}).to_string()
}

#[test]
fn jsonl_function_breakpoints_bind_metadata_and_apply_hit_policy() {
    let mut server = make_server();
    let initialized = server.handle_line(&request(1, "initialize", json!({})));
    assert_eq!(
        initialized[0]["body"]["capabilities"]["function_breakpoints"],
        true
    );
    let configured = server.handle_line(&request(
        2,
        "function_breakpoints.replace",
        json!({"breakpoints":[{"name":"HELPER","hit_condition":"2"}]}),
    ));
    let breakpoint = &configured[0]["body"]["breakpoints"][0];
    assert_eq!(breakpoint["verified"], true);
    assert_eq!(breakpoint["match_count"], 1);
    assert_eq!(breakpoint["matched_functions"], json!([1]));

    let _ = server.handle_line(&request(3, "launch", json!({"stop_on_entry":false})));
    let mut records = Vec::new();
    while server.status() == ServerStatus::Running {
        records.extend(server.wait());
    }
    let stopped = records
        .iter()
        .find(|record| record["event"] == "stopped")
        .expect("second function hit stops");
    assert_eq!(
        stopped["body"]["breakpoint_ids"],
        json!([breakpoint["breakpoint_id"]])
    );
}

#[test]
fn invalid_replace_is_atomic_and_missing_selector_is_unverified() {
    let mut server = make_server();
    let _ = server.handle_line(&request(1, "initialize", json!({})));
    let initial = server.handle_line(&request(
        2,
        "function_breakpoints.replace",
        json!({"breakpoints":[{"name":"Helper"}]}),
    ));
    let initial_id = initial[0]["body"]["breakpoints"][0]["breakpoint_id"].clone();
    let rejected = server.handle_line(&request(
        3,
        "function_breakpoints.replace",
        json!({"breakpoints":[{"name":"Helper","hit_condition":"0"}]}),
    ));
    assert_eq!(rejected[0]["success"], false);
    assert_eq!(rejected[0]["error"]["code"], "invalid_request");
    let _ = server.handle_line(&request(4, "launch", json!({"stop_on_entry":false})));
    let mut records = Vec::new();
    while server.status() == ServerStatus::Running {
        records.extend(server.wait());
    }
    let stopped = records
        .into_iter()
        .find(|record| record["event"] == "stopped")
        .expect("previous breakpoint remains active");
    assert_eq!(stopped["body"]["breakpoint_ids"], json!([initial_id]));

    let mut missing_server = make_server();
    let _ = missing_server.handle_line(&request(1, "initialize", json!({})));
    let missing = missing_server.handle_line(&request(
        2,
        "function_breakpoints.replace",
        json!({"breakpoints":[{"name":"Missing"}]}),
    ));
    assert_eq!(missing[0]["body"]["breakpoints"][0]["verified"], false);
    assert_eq!(missing[0]["body"]["breakpoints"][0]["match_count"], 0);

    let unsupported = missing_server.handle_line(&request(
        3,
        "function_breakpoints.replace",
        json!({"breakpoints":[{"name":"Helper","action":"mutate"}]}),
    ));
    assert_eq!(unsupported[0]["success"], false);
    assert_eq!(unsupported[0]["error"]["code"], "invalid_request");
}

#[test]
fn function_breakpoint_condition_uses_the_shared_read_only_policy() {
    let mut server = make_server();
    let _ = server.handle_line(&request(1, "initialize", json!({})));
    let _ = server.handle_line(&request(
        2,
        "function_breakpoints.replace",
        json!({"breakpoints":[{"name":"Helper","condition":"Value = 2"}]}),
    ));
    let _ = server.handle_line(&request(3, "launch", json!({"stop_on_entry":false})));
    let mut records = Vec::new();
    while server.status() == ServerStatus::Running {
        records.extend(server.wait());
    }
    let stopped = records
        .iter()
        .find(|record| record["event"] == "stopped")
        .expect("condition stops at the second call");
    assert_eq!(stopped["body"]["reason"], "breakpoint");
}

#[test]
fn same_boundary_source_log_precedes_the_single_function_stop() {
    let mut server = make_server();
    let _ = server.handle_line(&request(1, "initialize", json!({})));
    let configured = server.handle_line(&request(
        2,
        "function_breakpoints.replace",
        json!({"breakpoints":[{"name":"Helper"}]}),
    ));
    let function = &configured[0]["body"]["breakpoints"][0];
    let location = &function["locations"][0];
    let source = location["source"].as_str().expect("function source");
    let line = location["line"].as_u64().expect("function line");
    let source_log = server.handle_line(&request(
        3,
        "breakpoint.set",
        json!({"source":source,"line":line,"log_message":"enter Helper"}),
    ));
    let source_id = source_log[0]["body"]["breakpoint_id"].clone();
    let function_id = function["breakpoint_id"].clone();
    assert_eq!(source_log[0]["body"]["verified"], true, "{source_log:?}");

    let _ = server.handle_line(&request(4, "launch", json!({"stop_on_entry":false})));
    let mut records = Vec::new();
    while server.status() == ServerStatus::Running {
        records.extend(server.wait());
    }
    let output = records
        .iter()
        .position(|record| record["event"] == "output")
        .expect("source log output");
    let stop = records
        .iter()
        .position(|record| record["event"] == "stopped")
        .expect("function stop");
    assert!(output < stop, "{records:?}");
    assert_eq!(records[output]["body"]["text"], "enter Helper\n");
    assert_eq!(
        records[stop]["body"]["breakpoint_ids"],
        json!([function_id, source_id])
    );
}
