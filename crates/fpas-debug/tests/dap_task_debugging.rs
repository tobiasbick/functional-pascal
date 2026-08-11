//! DAP task-to-thread mapping and all-stop event coverage.

#![allow(
    clippy::expect_used,
    reason = "protocol tests use expect to keep fixture failures local"
)]

use fpas_debug::{DebugSourceContent, PreparedDebugTarget, dap::DapServer};
use serde_json::{Value, json};

const SOURCE: &str = r#"program TaskDebugging;

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

fn server() -> DapServer {
    let (program, diagnostics) = fpas_parser::parse(SOURCE);
    assert!(diagnostics.is_empty(), "parse diagnostics: {diagnostics:?}");
    let executable = fpas_compiler::compile(&program).expect("compile task fixture");
    DapServer::new(
        PreparedDebugTarget::new(executable, Vec::new()).with_sources(vec![DebugSourceContent {
            path: "<memory>".into(),
            content: SOURCE.into(),
        }]),
    )
    .expect("DAP server")
}

fn request(seq: u64, command: &str, arguments: Value) -> Value {
    json!({"seq":seq,"type":"request","command":command,"arguments":arguments})
}

#[test]
fn threads_stack_and_step_use_the_child_thread_identity() {
    let mut adapter = server();
    let initialized = adapter.handle(request(1, "initialize", json!({})));
    assert_eq!(
        initialized[0]["body"]["supportsSingleThreadExecutionRequests"],
        false
    );
    let _ = adapter.handle(request(2, "launch", json!({"stopOnEntry":false})));
    let breakpoint = adapter.handle(request(
        3,
        "setBreakpoints",
        json!({"source":{"path":"<memory>"},"breakpoints":[{"line":8}]}),
    ));
    assert_eq!(breakpoint[0]["body"]["breakpoints"][0]["verified"], true);
    let _ = adapter.handle(request(4, "configurationDone", json!({})));

    let events = adapter.wait();
    let child_thread = events
        .iter()
        .find(|event| event["event"] == "stopped")
        .and_then(|event| event["body"]["threadId"].as_u64())
        .expect("child stopped thread");
    assert_ne!(child_thread, 1);
    assert!(events.iter().any(|event| {
        event["event"] == "thread"
            && event["body"]["reason"] == "started"
            && event["body"]["threadId"] == child_thread
    }));

    let threads = adapter.handle(request(5, "threads", json!({})));
    let ids = threads[0]["body"]["threads"]
        .as_array()
        .expect("DAP threads")
        .iter()
        .filter_map(|thread| thread["id"].as_u64())
        .collect::<Vec<_>>();
    assert_eq!(ids, vec![1, child_thread]);

    let stack = adapter.handle(request(6, "stackTrace", json!({"threadId":child_thread})));
    assert_eq!(stack[0]["body"]["stackFrames"][0]["name"], "work");

    let step = adapter.handle(request(7, "stepIn", json!({"threadId":child_thread})));
    assert_eq!(step[0]["success"], true, "{step:?}");
    let stepped = adapter.wait();
    assert!(stepped.iter().any(|event| {
        event["event"] == "stopped"
            && event["body"]["reason"] == "step"
            && event["body"]["threadId"] == child_thread
            && event["body"]["allThreadsStopped"] == true
    }));

    let continued = adapter.handle(request(8, "continue", json!({"threadId":child_thread})));
    assert_eq!(continued[0]["body"]["allThreadsContinued"], true);
    let terminated = adapter.wait();
    assert_eq!(
        terminated
            .iter()
            .filter(|event| {
                event["event"] == "thread"
                    && event["body"]["reason"] == "exited"
                    && event["body"]["threadId"] == child_thread
            })
            .count(),
        1
    );
    assert!(
        terminated
            .iter()
            .any(|event| event["event"] == "terminated")
    );
}

#[test]
fn unknown_thread_is_not_mapped_to_main() {
    let mut adapter = server();
    let _ = adapter.handle(request(1, "initialize", json!({})));
    let _ = adapter.handle(request(2, "launch", json!({"stopOnEntry":true})));
    let _ = adapter.handle(request(3, "configurationDone", json!({})));

    let stack = adapter.handle(request(4, "stackTrace", json!({"threadId":99})));

    assert_eq!(stack[0]["success"], false);
    assert!(
        stack[0]["message"]
            .as_str()
            .is_some_and(|message| message.contains("unknown or expired"))
    );
}
