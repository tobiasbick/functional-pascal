//! DAP live-image hot-reload freeze.

#![allow(
    clippy::expect_used,
    reason = "protocol tests keep fixture failures local"
)]

use fpas_debug::{DebugSourceContent, PreparedDebugTarget, ReloadedDebugTarget, dap::DapServer};
use serde_json::{Value, json};

fn server() -> DapServer {
    let (program, diagnostics) = fpas_parser::parse("program HotReload; begin end.");
    assert!(diagnostics.is_empty(), "parse diagnostics: {diagnostics:?}");
    let executable = fpas_compiler::compile(&program).expect("compile hot-reload fixture");
    DapServer::new(PreparedDebugTarget::new(executable, Vec::new())).expect("DAP server")
}

fn compile_reloadable(value: i64) -> fpas_bytecode::VerifiedExecutable {
    let source = format!(
        "program HotReload;\nfunction Helper(): integer;\nbegin\n  return {value}\nend;\nbegin\nend."
    );
    let (program, diagnostics) = fpas_parser::parse(&source);
    assert!(diagnostics.is_empty(), "parse diagnostics: {diagnostics:?}");
    fpas_compiler::compile(&program).expect("compile reloadable fixture")
}

fn reloadable_server() -> DapServer {
    let candidate = compile_reloadable(2);
    let target = PreparedDebugTarget::new(compile_reloadable(1), Vec::new())
        .with_sources(vec![DebugSourceContent {
            path: "test.fpas".to_string(),
            original_path: Some("C:\\Workspace\\before.fpas".into()),
            content: "before".to_string(),
        }])
        .with_reloader(move || {
            Ok(
                ReloadedDebugTarget::new(candidate.clone()).with_sources(vec![
                    DebugSourceContent {
                        path: "test.fpas".to_string(),
                        original_path: Some("D:\\Workspace\\after.fpas".into()),
                        content: "after".to_string(),
                    },
                ]),
            )
        });
    DapServer::new(target).expect("reloadable DAP server")
}

fn send(server: &mut DapServer, seq: &mut u64, command: &str, arguments: Value) -> Vec<Value> {
    *seq += 1;
    server.handle(json!({
        "seq":*seq,"type":"request","command":command,"arguments":arguments
    }))
}

#[test]
fn dap_hot_reload_is_not_advertised_and_does_not_apply_without_launch() {
    let mut server = server();
    let mut seq = 0;
    let initialized = send(&mut server, &mut seq, "initialize", json!({}));
    assert_eq!(initialized[0]["body"]["supportsStepBack"], false);
    assert!(
        initialized[0]["body"].get("supportsHotReload").is_none(),
        "{initialized:?}"
    );

    let rejected = send(&mut server, &mut seq, "fpas/reload", json!({}));
    assert_eq!(rejected[0]["success"], true, "{rejected:?}");
    assert_eq!(rejected[0]["body"]["class"], "unchanged");
    assert_eq!(rejected[0]["body"]["applied"], false);
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

    let replaced = send(&mut server, &mut seq, "fpas/reload", json!({}));
    assert_eq!(replaced[0]["success"], true, "{replaced:?}");
    assert_eq!(replaced[0]["body"]["class"], "unchanged");
    assert_eq!(replaced[0]["body"]["applied"], false);
    assert_eq!(replaced.len(), 1, "{replaced:?}");

    let same_stack = send(&mut server, &mut seq, "stackTrace", json!({"threadId":1}));
    assert_eq!(same_stack[0]["body"]["stackFrames"][0]["id"], frame);
}

#[test]
fn dap_reload_classify_names_classes_without_replacing_the_image() {
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

    let classified = send(&mut server, &mut seq, "fpas/reloadClassify", json!({}));
    assert_eq!(classified[0]["success"], true, "{classified:?}");
    assert_eq!(classified[0]["body"]["class"], "unchanged");
    assert_eq!(classified[0]["body"]["accepted"], true);
    assert_eq!(classified[0]["body"]["applied"], false);
    assert_eq!(
        classified[0]["body"]["acceptedClasses"],
        json!(["unchanged", "inactive_function_body"])
    );

    let same_stack = send(&mut server, &mut seq, "stackTrace", json!({"threadId":1}));
    assert_eq!(same_stack[0]["body"]["stackFrames"][0]["id"], frame);
}

#[test]
fn dap_incompatible_replace_is_rejected_before_the_image_changes() {
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

    let (program, diagnostics) = fpas_parser::parse(
        "program IncompatibleReload;\nfunction Extra(): integer;\nbegin\n  return 1\nend;\nbegin\nend.",
    );
    assert!(diagnostics.is_empty(), "parse diagnostics: {diagnostics:?}");
    let candidate = fpas_compiler::compile(&program).expect("compile incompatible fixture");
    let error = server
        .replace_live_image(&candidate)
        .expect_err("incompatible replace");
    assert_eq!(error.kind, fpas_vm::DebugErrorKind::LiveImageIncompatible);

    let same_stack = send(&mut server, &mut seq, "stackTrace", json!({"threadId":1}));
    assert_eq!(same_stack[0]["body"]["stackFrames"][0]["id"], frame);
}

#[test]
fn dap_reload_and_rollback_match_jsonl_and_refresh_sources() {
    let mut server = reloadable_server();
    let mut seq = 0;
    let initialized = send(
        &mut server,
        &mut seq,
        "initialize",
        json!({"supportsInvalidatedEvent":true}),
    );
    assert_eq!(initialized[0]["body"]["supportsHotReload"], true);
    let _ = send(&mut server, &mut seq, "launch", json!({"stopOnEntry":true}));
    let _ = send(&mut server, &mut seq, "configurationDone", json!({}));
    let _ = server.wait();

    let reloaded = send(&mut server, &mut seq, "fpas/reload", json!({}));
    assert_eq!(reloaded[0]["success"], true, "{reloaded:?}");
    assert_eq!(reloaded[0]["body"]["class"], "inactive_function_body");
    assert_eq!(reloaded[0]["body"]["applied"], true);
    assert_eq!(reloaded[0]["body"]["version"], 2);
    assert_eq!(reloaded[0]["body"]["rollbackAvailable"], true);
    assert!(
        reloaded
            .iter()
            .any(|message| message["event"] == "invalidated"),
        "{reloaded:?}"
    );
    let source = send(
        &mut server,
        &mut seq,
        "source",
        json!({"source":{"path":"d:/workspace/AFTER.fpas"}}),
    );
    assert_eq!(source[0]["body"]["content"], "after");

    let rolled_back = send(&mut server, &mut seq, "fpas/reloadRollback", json!({}));
    assert_eq!(rolled_back[0]["success"], true, "{rolled_back:?}");
    assert_eq!(rolled_back[0]["body"]["applied"], true);
    assert_eq!(rolled_back[0]["body"]["version"], 3);
    let source = send(
        &mut server,
        &mut seq,
        "source",
        json!({"source":{"path":"c:/workspace/BEFORE.fpas"}}),
    );
    assert_eq!(source[0]["body"]["content"], "before");
}
