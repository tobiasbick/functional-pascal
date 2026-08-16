//! DAP rejected `goto` / `gotoTargets` coverage.

#![allow(
    clippy::expect_used,
    reason = "protocol tests keep fixture failures local"
)]

use fpas_debug::{PreparedDebugTarget, dap::DapServer};
use serde_json::{Value, json};

const SOURCE: &str = r#"program InstructionChange;

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
    let executable = fpas_compiler::compile(&program).expect("compile instruction-change fixture");
    DapServer::new(PreparedDebugTarget::new(executable, Vec::new())).expect("DAP server")
}

fn send(server: &mut DapServer, seq: &mut u64, command: &str, arguments: Value) -> Vec<Value> {
    *seq += 1;
    server.handle(json!({
        "seq":*seq,"type":"request","command":command,"arguments":arguments
    }))
}

#[test]
fn dap_goto_is_not_advertised_and_maps_the_shared_rejection() {
    let mut server = server();
    let mut seq = 0;
    let initialized = send(&mut server, &mut seq, "initialize", json!({}));
    assert_eq!(initialized[0]["body"]["supportsGotoTargetsRequest"], false);
    assert_eq!(initialized[0]["body"]["supportsRestartFrame"], true);
    let _ = send(&mut server, &mut seq, "launch", json!({"stopOnEntry":true}));
    let _ = send(&mut server, &mut seq, "configurationDone", json!({}));
    let _ = server.wait();
    let stack = send(&mut server, &mut seq, "stackTrace", json!({"threadId":1}));
    let frame = stack[0]["body"]["stackFrames"][0]["id"]
        .as_u64()
        .expect("entry frame");

    let goto_targets = send(
        &mut server,
        &mut seq,
        "gotoTargets",
        json!({"source":{"path":"<memory>"},"line":8}),
    );
    assert_eq!(goto_targets[0]["success"], false, "{goto_targets:?}");
    assert_eq!(
        goto_targets[0]["body"]["error"]["code"],
        "instruction_change_unsupported"
    );

    let goto = send(
        &mut server,
        &mut seq,
        "goto",
        json!({"threadId":1,"targetId":1,"frameId":frame}),
    );
    assert_eq!(goto[0]["success"], false, "{goto:?}");
    assert_eq!(
        goto[0]["body"]["error"]["code"],
        "instruction_change_unsupported"
    );
    assert_eq!(goto.len(), 1, "{goto:?}");

    let same_stack = send(&mut server, &mut seq, "stackTrace", json!({"threadId":1}));
    assert_eq!(same_stack[0]["body"]["stackFrames"][0]["id"], frame);
}
