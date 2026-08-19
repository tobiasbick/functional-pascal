//! DAP all-stop thread identity and continue/pause session scope.

#![allow(
    clippy::expect_used,
    reason = "protocol tests use expect to keep fixture failures local"
)]

use fpas_debug::{DebugSourceContent, PreparedDebugTarget, dap::DapServer, jsonl::JsonlServer};
use serde_json::{Value, json};

const SOURCE: &str = r#"program TaskQuiescence;

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
    let executable = fpas_compiler::compile(&program).expect("compile quiescence fixture");
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

#[test]
fn stopped_events_and_continue_are_session_wide() {
    let mut adapter = dap_server();
    let initialized = adapter.handle(request(1, "initialize", json!({})));
    assert_eq!(
        initialized[0]["body"]["supportsSingleThreadExecutionRequests"],
        false
    );
    let _ = adapter.handle(request(2, "launch", json!({"stopOnEntry":false})));
    let _ = adapter.handle(request(
        3,
        "setBreakpoints",
        json!({"source":{"path":"<memory>"},"breakpoints":[{"line":8}]}),
    ));
    let _ = adapter.handle(request(4, "configurationDone", json!({})));

    let events = adapter.wait();
    let stopped = events
        .iter()
        .find(|event| event["event"] == "stopped")
        .expect("child stopped");
    let child_thread = stopped["body"]["threadId"].as_u64().expect("thread id");
    assert_ne!(child_thread, 1);
    assert_eq!(stopped["body"]["allThreadsStopped"], true);

    let threads = adapter.handle(request(5, "threads", json!({})));
    let ids = threads[0]["body"]["threads"]
        .as_array()
        .expect("DAP threads")
        .iter()
        .filter_map(|thread| thread["id"].as_u64())
        .collect::<Vec<_>>();
    assert_eq!(ids, vec![1, child_thread]);

    let continued = adapter.handle(request(
        6,
        "continue",
        json!({"threadId":child_thread,"singleThread":true}),
    ));
    assert_eq!(continued[0]["success"], true, "{continued:?}");
    assert_eq!(continued[0]["body"]["allThreadsContinued"], true);
    let terminated = adapter.wait();
    assert!(
        terminated
            .iter()
            .any(|event| event["event"] == "terminated")
    );
}

#[test]
fn jsonl_task_ids_map_to_stable_dap_threads() {
    let mut jsonl = JsonlServer::new(target()).expect("JSONL server");
    let _ = jsonl.handle_line(&jsonl_request(1, "initialize", json!({"version":2})));
    let _ = jsonl.handle_line(&jsonl_request(
        2,
        "breakpoint.set",
        json!({"source":"<memory>","line":8}),
    ));
    let _ = jsonl.handle_line(&jsonl_request(3, "launch", json!({"stop_on_entry":false})));
    let jsonl_events = jsonl.wait();
    let jsonl_stop = jsonl_events
        .iter()
        .find(|record| record["event"] == "stopped")
        .expect("JSONL stop");
    assert_eq!(jsonl_stop["body"]["task_id"], 1);
    assert_eq!(jsonl_stop["body"]["all_tasks_stopped"], true);
    let jsonl_tasks = jsonl.handle_line(&jsonl_request(4, "tasks", json!({})));
    assert_eq!(jsonl_tasks[0]["body"]["tasks"][0]["task_id"], 0);
    assert_eq!(jsonl_tasks[0]["body"]["tasks"][1]["task_id"], 1);

    let mut dap = dap_server();
    let _ = dap.handle(request(1, "initialize", json!({})));
    let _ = dap.handle(request(2, "launch", json!({"stopOnEntry":false})));
    let _ = dap.handle(request(
        3,
        "setBreakpoints",
        json!({"source":{"path":"<memory>"},"breakpoints":[{"line":8}]}),
    ));
    let _ = dap.handle(request(4, "configurationDone", json!({})));
    let dap_events = dap.wait();
    let dap_stop = dap_events
        .iter()
        .find(|event| event["event"] == "stopped")
        .expect("DAP stop");
    assert_eq!(dap_stop["body"]["allThreadsStopped"], true);
    let child_thread = dap_stop["body"]["threadId"].as_u64().expect("child thread");
    assert_ne!(child_thread, 1);

    let threads = dap.handle(request(5, "threads", json!({})));
    let names = threads[0]["body"]["threads"]
        .as_array()
        .expect("DAP threads")
        .iter()
        .map(|thread| {
            (
                thread["id"].as_u64().expect("id"),
                thread["name"].as_str().unwrap_or("").to_string(),
            )
        })
        .collect::<Vec<_>>();
    assert_eq!(names[0], (1, "FPAS main".to_string()));
    assert_eq!(names[1].0, child_thread);
    assert!(names[1].1.contains("1"), "{names:?}");
}
