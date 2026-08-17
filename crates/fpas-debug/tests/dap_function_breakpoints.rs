//! DAP function-breakpoint capability and stop mapping.

#![allow(
    clippy::expect_used,
    reason = "DAP fixtures use expect to keep transcript failures local"
)]

use fpas_debug::{PreparedDebugTarget, dap::DapServer};
use serde_json::{Value, json};

fn server() -> DapServer {
    let source = r#"program Main;

function Helper(Value: integer): integer;
begin
  return Value + 1
end;

begin
  var First: integer := Helper(1);
  var Second: integer := Helper(First)
end.
"#;
    let (program, diagnostics) = fpas_parser::parse(source);
    assert!(diagnostics.is_empty(), "{diagnostics:?}");
    let executable = fpas_compiler::compile(&program).expect("compile function fixture");
    DapServer::new(PreparedDebugTarget::new(executable, Vec::new())).expect("create DAP server")
}

fn request(seq: u64, command: &str, arguments: Value) -> Value {
    json!({"seq":seq,"type":"request","command":command,"arguments":arguments})
}

#[test]
fn dap_function_breakpoint_uses_standard_request_and_second_hit_stop() {
    let mut server = server();
    let initialized = server.handle(request(1, "initialize", json!({})));
    assert_eq!(initialized[0]["body"]["supportsFunctionBreakpoints"], true);
    let _ = server.handle(request(2, "launch", json!({"stopOnEntry":false})));
    let configured = server.handle(request(
        3,
        "setFunctionBreakpoints",
        json!({"breakpoints":[{"name":"Helper","hitCondition":"2"}]}),
    ));
    assert_eq!(configured[0]["success"], true);
    assert_eq!(configured[0]["body"]["breakpoints"][0]["verified"], true);
    assert!(
        configured[0]["body"]["breakpoints"][0]["id"]
            .as_u64()
            .is_some()
    );

    let started = server.handle(request(4, "configurationDone", json!({})));
    assert!(started.iter().any(|message| {
        message["command"] == "configurationDone" && message["success"] == true
    }));
    let mut messages = Vec::new();
    while server.is_running() {
        messages.extend(server.wait());
    }
    assert!(messages.iter().any(|message| {
        message["event"] == "stopped" && message["body"]["reason"] == "breakpoint"
    }));
}

#[test]
fn dap_missing_function_selector_is_unverified() {
    let mut server = server();
    let _ = server.handle(request(1, "initialize", json!({})));
    let configured = server.handle(request(
        2,
        "setFunctionBreakpoints",
        json!({"breakpoints":[{"name":"Missing"}]}),
    ));

    assert_eq!(configured[0]["success"], true);
    assert_eq!(configured[0]["body"]["breakpoints"][0]["verified"], false);
    assert!(
        configured[0]["body"]["breakpoints"][0]["message"]
            .as_str()
            .is_some_and(|message| message.contains("No executable function"))
    );
}

#[test]
fn dap_rejects_unsupported_function_logpoints_and_actions() {
    for unsupported in [
        json!({"logMessage":"value"}),
        json!({"action":"mutate"}),
        json!({"assign":{"identity":{"index":0},"expression":"1"}}),
    ] {
        let mut server = server();
        let _ = server.handle(request(1, "initialize", json!({})));
        let mut breakpoint = json!({"name":"Helper"});
        breakpoint
            .as_object_mut()
            .expect("breakpoint object")
            .extend(unsupported.as_object().expect("unsupported field").clone());
        let response = server.handle(request(
            2,
            "setFunctionBreakpoints",
            json!({"breakpoints":[breakpoint]}),
        ));
        assert_eq!(response[0]["success"], false);
    }
}
