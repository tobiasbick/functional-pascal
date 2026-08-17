//! DAP data-breakpoint freeze until durable location identities exist.

#![allow(
    clippy::expect_used,
    reason = "protocol tests keep fixture failures local"
)]

use fpas_debug::{PreparedDebugTarget, dap::DapServer};
use serde_json::{Value, json};

fn server() -> DapServer {
    let (program, diagnostics) = fpas_parser::parse("program DataBreakpoints; begin end.");
    assert!(diagnostics.is_empty(), "parse diagnostics: {diagnostics:?}");
    let executable = fpas_compiler::compile(&program).expect("compile data-breakpoint fixture");
    DapServer::new(PreparedDebugTarget::new(executable, Vec::new())).expect("DAP server")
}

fn send(server: &mut DapServer, seq: &mut u64, command: &str, arguments: Value) -> Vec<Value> {
    *seq += 1;
    server.handle(json!({
        "seq":*seq,"type":"request","command":command,"arguments":arguments
    }))
}

#[test]
fn dap_data_breakpoints_are_not_advertised_and_reject_without_launch() {
    let mut server = server();
    let mut seq = 0;
    let initialized = send(&mut server, &mut seq, "initialize", json!({}));
    assert_eq!(initialized[0]["body"]["supportsDataBreakpoints"], false);

    for command in ["setDataBreakpoints", "dataBreakpointInfo"] {
        let rejected = send(&mut server, &mut seq, command, json!({}));
        assert_eq!(rejected[0]["success"], false, "{command}: {rejected:?}");
        assert!(
            rejected[0]["message"]
                .as_str()
                .is_some_and(|message| message.contains("Data breakpoints")),
            "{command}: {rejected:?}"
        );
        assert_eq!(rejected.len(), 1, "{command}: {rejected:?}");
    }

    let _ = send(&mut server, &mut seq, "launch", json!({"stopOnEntry":true}));
    let configured = send(&mut server, &mut seq, "configurationDone", json!({}));
    assert!(
        configured
            .iter()
            .any(|message| message["event"] == "stopped"),
        "{configured:?}"
    );
}

#[test]
fn dap_data_breakpoint_reject_after_stop_does_not_resume() {
    let mut server = server();
    let mut seq = 0;
    let _ = send(&mut server, &mut seq, "initialize", json!({}));
    let _ = send(&mut server, &mut seq, "launch", json!({"stopOnEntry":true}));
    let _ = send(&mut server, &mut seq, "configurationDone", json!({}));
    let _ = server.wait();
    let stack = send(&mut server, &mut seq, "stackTrace", json!({"threadId":1}));
    let frame = stack[0]["body"]["stackFrames"][0]["id"]
        .as_u64()
        .expect("entry frame");

    for command in ["setDataBreakpoints", "dataBreakpointInfo"] {
        let rejected = send(&mut server, &mut seq, command, json!({}));
        assert_eq!(rejected[0]["success"], false, "{command}: {rejected:?}");
        assert_eq!(rejected.len(), 1, "{command}: {rejected:?}");
    }

    let same_stack = send(&mut server, &mut seq, "stackTrace", json!({"threadId":1}));
    assert_eq!(same_stack[0]["body"]["stackFrames"][0]["id"], frame);
}
