//! JSONL variable-mutation capability, success, failure, and generation contracts.

#![allow(
    clippy::expect_used,
    reason = "protocol tests keep fixture failures local"
)]

use std::{thread, time::Duration};

use fpas_debug::{PreparedDebugTarget, jsonl::JsonlServer};
use fpas_vm::{
    DebugAssignmentSelector, DebugAssignmentTarget, DebugErrorKind, DebugEvaluationLimits,
    DebugExpression, DebugRunResult, DebugSession,
};
use serde_json::{Value, json};

fn server() -> JsonlServer {
    let source = r#"program Main;

function Twice(Value: integer): integer;
begin
  return Value * 2;
end function;

begin
   var X: integer := 1;
  const Fixed: integer := 2;
  X := X + Fixed;
end program;"#;
    let (program, diagnostics) = fpas_parser::parse(source);
    assert!(diagnostics.is_empty(), "parse diagnostics: {diagnostics:?}");
    let executable = fpas_compiler::compile(&program).expect("compile mutation fixture");
    JsonlServer::new(PreparedDebugTarget::new(executable, Vec::new())).expect("JSONL server")
}

fn request(id: u64, command: &str, arguments: Value) -> String {
    json!({"type":"request","id":id,"command":command,"arguments":arguments}).to_string()
}

fn session(source: &str) -> DebugSession {
    let (program, diagnostics) = fpas_parser::parse(source);
    assert!(diagnostics.is_empty(), "parse diagnostics: {diagnostics:?}");
    let executable = fpas_compiler::compile(&program).expect("compile mutation fixture");
    DebugSession::new(executable).expect("debug session")
}

fn session_scope(session: &mut DebugSession, name: &str) -> Option<u64> {
    let frame = session.stack(0, 1).ok()?.items.first()?.id;
    session
        .scopes(frame)
        .ok()?
        .into_iter()
        .find(|scope| scope.name == name)
        .map(|scope| scope.variables_reference)
}

fn step(session: &mut DebugSession) {
    assert!(matches!(
        session.step_into().expect("step"),
        DebugRunResult::Stopped(_)
    ));
}

fn locals_reference(server: &mut JsonlServer, id: &mut u64) -> u64 {
    *id += 1;
    let stack = server.handle_line(&request(*id, "stack", json!({})));
    let frame = stack[0]["body"]["frames"][0]["frame_id"]
        .as_u64()
        .expect("frame");
    *id += 1;
    let scopes = server.handle_line(&request(*id, "scopes", json!({"frame_id":frame})));
    scopes[0]["body"]["scopes"]
        .as_array()
        .expect("scopes")
        .iter()
        .find(|scope| scope["name"] == "Locals")
        .and_then(|scope| scope["variables_reference"].as_u64())
        .expect("locals")
}

#[test]
fn jsonl_sets_variables_with_stable_errors_and_fresh_handles() {
    let mut server = server();
    let initialized = server.handle_line(&request(1, "initialize", json!({"version":2})));
    assert_eq!(initialized[0]["body"]["capabilities"]["set_variable"], true);
    let _ = server.handle_line(&request(2, "launch", json!({"stop_on_entry":true})));
    let _ = server.handle_line(&request(3, "step_into", json!({})));
    let _ = server.wait();

    let mut id = 3;
    let locals = locals_reference(&mut server, &mut id);
    let variables = server.handle_line(&request(
        {
            id += 1;
            id
        },
        "variables",
        json!({"variables_reference":locals}),
    ));
    assert_eq!(variables[0]["body"]["variables"][0]["value"], "1");

    let missing = server.handle_line(&request(
        {
            id += 1;
            id
        },
        "variable.set",
        json!({"variables_reference":locals,"name":"X"}),
    ));
    assert_eq!(missing[0]["error"]["code"], "invalid_request");

    let unknown = server.handle_line(&request(
        {
            id += 1;
            id
        },
        "variable.set",
        json!({"variables_reference":locals,"name":"Missing","expression":"1"}),
    ));
    assert_eq!(unknown[0]["error"]["code"], "variable_target_unknown");

    let wrong = server.handle_line(&request(
        {
            id += 1;
            id
        },
        "variable.set",
        json!({"variables_reference":locals,"name":"X","expression":"'wrong'"}),
    ));
    assert_eq!(wrong[0]["error"]["code"], "variable_value_type");

    let updated = server.handle_line(&request(
        {
            id += 1;
            id
        },
        "variable.set",
        json!({"variables_reference":locals,"name":"X","expression":"Twice(21)"}),
    ));
    assert_eq!(updated[0]["body"]["result"], "42", "{updated:?}");

    let expired = server.handle_line(&request(
        {
            id += 1;
            id
        },
        "variable.set",
        json!({"variables_reference":locals,"name":"X","expression":"1"}),
    ));
    assert_eq!(expired[0]["error"]["code"], "variable_target_expired");

    let locals = locals_reference(&mut server, &mut id);
    let fixed = server.handle_line(&request(
        {
            id += 1;
            id
        },
        "variable.set",
        json!({"variables_reference":locals,"name":"Fixed","expression":"3"}),
    ));
    assert_eq!(fixed[0]["error"]["code"], "variable_not_mutable");

    let variables = server.handle_line(&request(
        {
            id += 1;
            id
        },
        "variables",
        json!({"variables_reference":locals}),
    ));
    assert_eq!(variables[0]["body"]["variables"][0]["value"], "42");
}

#[path = "variable_mutation/limits.rs"]
mod limits;
#[path = "variable_mutation/references.rs"]
mod references;
#[path = "variable_mutation/storage_roots.rs"]
mod storage_roots;
