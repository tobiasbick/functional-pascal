//! DAP live-image hot-reload freeze.

#![allow(
    clippy::expect_used,
    reason = "protocol tests keep fixture failures local"
)]

use fpas_debug::{PreparedDebugTarget, dap::DapServer};
use serde_json::{Value, json};

fn server() -> DapServer {
    let (program, diagnostics) = fpas_parser::parse("program HotReload; begin end.");
    assert!(diagnostics.is_empty(), "parse diagnostics: {diagnostics:?}");
    let executable = fpas_compiler::compile(&program).expect("compile hot-reload fixture");
    DapServer::new(PreparedDebugTarget::new(executable, Vec::new())).expect("DAP server")
}

fn send(server: &mut DapServer, seq: &mut u64, command: &str, arguments: Value) -> Vec<Value> {
    *seq += 1;
    server.handle(json!({
        "seq":*seq,"type":"request","command":command,"arguments":arguments
    }))
}

#[test]
fn dap_hot_reload_is_not_advertised_and_rejects_without_launch() {
    let mut server = server();
    let mut seq = 0;
    let initialized = send(&mut server, &mut seq, "initialize", json!({}));
    assert_eq!(initialized[0]["body"]["supportsStepBack"], false);
    assert!(
        initialized[0]["body"].get("supportsHotReload").is_none(),
        "{initialized:?}"
    );

    let rejected = send(&mut server, &mut seq, "fpas/reload", json!({}));
    assert_eq!(rejected[0]["success"], false, "{rejected:?}");
    assert!(
        rejected[0]["message"]
            .as_str()
            .is_some_and(|message| message.contains("hot reload")),
        "{rejected:?}"
    );
    assert_eq!(rejected.len(), 1, "{rejected:?}");

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
fn dap_hot_reload_after_stop_does_not_resume_or_replace_the_image() {
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

    let _ = send(&mut server, &mut seq, "fpas/record", json!({}));
    let described = send(&mut server, &mut seq, "fpas/recordingDescribe", json!({}));
    assert_eq!(described[0]["body"]["replayable"], false);

    let rejected = send(&mut server, &mut seq, "fpas/reload", json!({}));
    assert_eq!(rejected[0]["success"], false, "{rejected:?}");
    assert_eq!(rejected.len(), 1, "{rejected:?}");

    let same_stack = send(&mut server, &mut seq, "stackTrace", json!({"threadId":1}));
    assert_eq!(same_stack[0]["body"]["stackFrames"][0]["id"], frame);
}
