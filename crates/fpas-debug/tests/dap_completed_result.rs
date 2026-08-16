//! DAP retained completed-task result replacement coverage.

#![allow(
    clippy::expect_used,
    reason = "protocol tests keep fixture failures local"
)]

use fpas_debug::{PreparedDebugTarget, dap::DapServer};
use serde_json::{Value, json};

const SOURCE: &str = r#"program CompletedTaskResult;

uses Std.Console, Std.Task;

function Work(): integer;
begin
  return 7
end;

begin
  var Pending: task := go Work();
  WriteLn(Wait(Pending))
end.
"#;

fn server() -> DapServer {
    let (program, diagnostics) = fpas_parser::parse(SOURCE);
    assert!(diagnostics.is_empty(), "parse diagnostics: {diagnostics:?}");
    let executable = fpas_compiler::compile(&program).expect("compile completed-result fixture");
    DapServer::new(PreparedDebugTarget::new(executable, Vec::new())).expect("DAP server")
}

fn send(server: &mut DapServer, seq: &mut u64, command: &str, arguments: Value) -> Vec<Value> {
    *seq += 1;
    server.handle(json!({
        "seq": *seq,
        "type": "request",
        "command": command,
        "arguments": arguments
    }))
}

#[test]
fn dap_replaces_completed_result_and_invalidates_only_variables() {
    let mut server = server();
    let mut seq = 0;
    let _ = send(
        &mut server,
        &mut seq,
        "initialize",
        json!({"supportsInvalidatedEvent":true}),
    );
    let breakpoint = send(
        &mut server,
        &mut seq,
        "setBreakpoints",
        json!({"source":{"path":"<memory>"},"breakpoints":[{"line":7}]}),
    );
    assert_eq!(breakpoint[0]["body"]["breakpoints"][0]["verified"], true);
    let _ = send(
        &mut server,
        &mut seq,
        "launch",
        json!({"stopOnEntry":false}),
    );
    let _ = send(&mut server, &mut seq, "configurationDone", json!({}));
    let stopped = server.wait();
    let child_thread = stopped
        .iter()
        .find(|record| record["event"] == "stopped")
        .and_then(|record| record["body"]["threadId"].as_u64())
        .expect("child thread");
    let child_stack = send(
        &mut server,
        &mut seq,
        "stackTrace",
        json!({"threadId":child_thread}),
    );
    let child_frame = child_stack[0]["body"]["stackFrames"][0]["id"]
        .as_u64()
        .expect("child frame");
    let completed = send(
        &mut server,
        &mut seq,
        "fpas/forceReturn",
        json!({"frameId":child_frame,"expression":"7"}),
    );
    assert_eq!(completed[0]["success"], true, "{completed:?}");

    let root_stack = send(&mut server, &mut seq, "stackTrace", json!({"threadId":1}));
    let root_frame = root_stack[0]["body"]["stackFrames"][0]["id"]
        .as_u64()
        .expect("root frame");
    let replaced = send(
        &mut server,
        &mut seq,
        "fpas/replaceTaskResult",
        json!({"taskId":1,"frameId":root_frame,"expression":"9"}),
    );
    assert_eq!(replaced[0]["success"], true, "{replaced:?}");
    assert_eq!(replaced[0]["body"]["taskId"], 1);
    assert_eq!(replaced[0]["body"]["value"], "9");
    assert_eq!(replaced[0]["body"]["type"], "integer");
    assert_eq!(replaced.len(), 2, "{replaced:?}");
    assert_eq!(replaced[1]["event"], "invalidated");
    assert_eq!(replaced[1]["body"]["areas"], json!(["variables"]));

    let stale = send(
        &mut server,
        &mut seq,
        "fpas/replaceTaskResult",
        json!({"taskId":1,"frameId":root_frame,"expression":"10"}),
    );
    assert_eq!(stale.len(), 1, "{stale:?}");
    assert_eq!(stale[0]["body"]["error"]["code"], "unknown_frame");
}
