//! DAP selected live-frame restart coverage.

#![allow(
    clippy::expect_used,
    reason = "protocol tests keep fixture failures local"
)]

use fpas_debug::{PreparedDebugTarget, dap::DapServer};
use serde_json::{Value, json};

const SOURCE: &str = r#"program FrameRestart;

uses Std.Console;

function Branch(Value: integer): integer;
begin
  mutable var Local: integer := Value + 10;
  WriteLn('effect');
  return Local
end;

begin
  WriteLn(Branch(1))
end.
"#;

fn server() -> DapServer {
    let (program, diagnostics) = fpas_parser::parse(SOURCE);
    assert!(diagnostics.is_empty(), "parse diagnostics: {diagnostics:?}");
    let executable = fpas_compiler::compile(&program).expect("compile restart fixture");
    DapServer::new(PreparedDebugTarget::new(executable, Vec::new())).expect("DAP server")
}

fn send(server: &mut DapServer, seq: &mut u64, command: &str, arguments: Value) -> Vec<Value> {
    *seq += 1;
    server.handle(json!({
        "seq":*seq,"type":"request","command":command,"arguments":arguments
    }))
}

#[test]
fn dap_restart_maps_the_standard_request_and_invalidates_stacks_and_variables() {
    let mut server = server();
    let mut seq = 0;
    let initialized = send(
        &mut server,
        &mut seq,
        "initialize",
        json!({"supportsInvalidatedEvent":true}),
    );
    assert_eq!(initialized[0]["body"]["supportsRestartFrame"], true);
    let _ = send(
        &mut server,
        &mut seq,
        "setBreakpoints",
        json!({"source":{"path":"<memory>"},"breakpoints":[{"line":9}]}),
    );
    let _ = send(
        &mut server,
        &mut seq,
        "launch",
        json!({"stopOnEntry":false}),
    );
    let _ = send(&mut server, &mut seq, "configurationDone", json!({}));
    let _ = server.wait();
    let stack = send(&mut server, &mut seq, "stackTrace", json!({"threadId":1}));
    let frame = stack[0]["body"]["stackFrames"][0]["id"]
        .as_u64()
        .expect("branch frame");

    let restarted = send(
        &mut server,
        &mut seq,
        "restartFrame",
        json!({"frameId":frame}),
    );
    assert_eq!(restarted[0]["success"], true, "{restarted:?}");
    assert_eq!(restarted[0]["body"]["taskId"], 0);
    assert_eq!(restarted[0]["body"]["discardedFrames"], 0);
    assert_eq!(restarted.len(), 2, "{restarted:?}");
    assert_eq!(restarted[1]["event"], "invalidated");
    assert_eq!(
        restarted[1]["body"]["areas"],
        json!(["stacks", "variables"])
    );

    let stale = send(
        &mut server,
        &mut seq,
        "restartFrame",
        json!({"frameId":frame}),
    );
    assert_eq!(stale.len(), 1, "{stale:?}");
    assert_eq!(stale[0]["body"]["error"]["code"], "unknown_frame");
}
