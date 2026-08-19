//! DAP task cancel plus rejected create/restart mapped through JSONL.

#![allow(
    clippy::expect_used,
    reason = "protocol tests use expect to keep fixture failures local"
)]

use fpas_debug::{DebugSourceContent, PreparedDebugTarget, dap::DapServer, jsonl::JsonlServer};
use serde_json::{Value, json};

const SOURCE: &str = r#"program TaskLifecycle;

uses Std.Task;

function Work(): integer;
begin
  mutable var Value: integer := 40;
  Value := Value + 2;
  return Value
end;

begin
  var Pending: task := go Work();
  Wait(Pending)
end.
"#;

fn target() -> PreparedDebugTarget {
    let (program, diagnostics) = fpas_parser::parse(SOURCE);
    assert!(diagnostics.is_empty(), "parse diagnostics: {diagnostics:?}");
    let executable = fpas_compiler::compile(&program).expect("compile task-lifecycle fixture");
    PreparedDebugTarget::new(executable, Vec::new()).with_sources(vec![DebugSourceContent {
        path: "<memory>".into(),
        original_path: None,
        content: SOURCE.into(),
    }])
}

fn dap_server() -> DapServer {
    DapServer::new(target()).expect("DAP server")
}

fn request(seq: u64, command: &str, arguments: Value) -> Value {
    json!({"seq":seq,"type":"request","command":command,"arguments":arguments})
}

fn jsonl_request(id: u64, command: &str, arguments: Value) -> String {
    json!({"type":"request","id":id,"command":command,"arguments":arguments}).to_string()
}

fn stop_child(adapter: &mut DapServer) -> u64 {
    let _ = adapter.handle(request(2, "launch", json!({"stopOnEntry":false})));
    let _ = adapter.handle(request(
        3,
        "setBreakpoints",
        json!({"source":{"path":"<memory>"},"breakpoints":[{"line":8}]}),
    ));
    let _ = adapter.handle(request(4, "configurationDone", json!({})));
    let events = adapter.wait();
    events
        .iter()
        .find(|event| event["event"] == "stopped")
        .and_then(|event| event["body"]["threadId"].as_u64())
        .expect("child stopped thread")
}

#[test]
fn cancel_task_exits_the_dap_thread_and_jsonl_catalog() {
    let mut adapter = dap_server();
    let initialized = adapter.handle(request(1, "initialize", json!({})));
    assert_eq!(
        initialized[0]["body"]["supportsSingleThreadExecutionRequests"],
        false
    );
    assert_eq!(initialized[0]["body"]["supportsStepBack"], false);
    let child_thread = stop_child(&mut adapter);
    assert_ne!(child_thread, 1);

    let cancelled = adapter.handle(request(
        5,
        "fpas/cancelTask",
        json!({"threadId": child_thread}),
    ));
    assert_eq!(cancelled[0]["success"], true, "{cancelled:?}");
    assert_eq!(cancelled[0]["body"]["state"], "cancelled");
    assert!(cancelled.iter().any(|record| {
        record["event"] == "thread"
            && record["body"]["reason"] == "exited"
            && record["body"]["threadId"] == child_thread
    }));

    let threads = adapter.handle(request(6, "threads", json!({})));
    let listed = threads[0]["body"]["threads"]
        .as_array()
        .expect("DAP threads");
    assert!(
        !listed.iter().any(|thread| thread["id"] == child_thread),
        "{listed:?}"
    );

    let unknown = adapter.handle(request(7, "fpas/cancelTask", json!({"threadId": 99})));
    assert_eq!(unknown[0]["success"], false, "{unknown:?}");

    let created = adapter.handle(request(8, "fpas/createTask", json!({})));
    assert_eq!(created[0]["success"], false, "{created:?}");
    let restarted = adapter.handle(request(
        9,
        "fpas/restartTask",
        json!({"threadId": child_thread}),
    ));
    assert_eq!(restarted[0]["success"], false, "{restarted:?}");

    let mut jsonl = JsonlServer::new(target()).expect("JSONL server");
    let _ = jsonl.handle_line(&jsonl_request(1, "initialize", json!({"version":2})));
    let _ = jsonl.handle_line(&jsonl_request(
        2,
        "breakpoint.set",
        json!({"source":"<memory>","line":8}),
    ));
    let _ = jsonl.handle_line(&jsonl_request(3, "launch", json!({"stop_on_entry":false})));
    let _ = jsonl.wait();
    let cancelled = jsonl.handle_line(&jsonl_request(4, "task.cancel", json!({"task_id":1})));
    assert_eq!(cancelled[0]["body"]["state"], "cancelled");
    let tasks = jsonl.handle_line(&jsonl_request(5, "tasks", json!({})));
    assert_eq!(tasks[0]["body"]["tasks"][1]["state"], "cancelled");
}
