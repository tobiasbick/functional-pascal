//! Watch parsing, controlled calls, skipped errors, and `try` short-circuit contracts.

#![allow(
    clippy::expect_used,
    clippy::panic,
    reason = "integration fixture failures stay local"
)]

use fpas_debug::{PreparedDebugTarget, jsonl::JsonlServer};
use serde_json::{Value, json};

fn request(id: u64, command: &str, arguments: Value) -> String {
    json!({"type":"request", "id":id, "command":command, "arguments":arguments}).to_string()
}

#[test]
fn watches_short_circuit_calls_errors_and_try_while_preserving_eager_operations() {
    let source = r#"
program DebugBoolean;
uses Std.Console, Std.Bits;
function Identity(Value: boolean): boolean;
begin return Value; end function;
function Denied(): boolean;
begin WriteLn('must not reach the live console'); return true; end function;
function Guarded(): boolean;
begin return false and (1 div 0 > 0); end function;
begin
end.
"#;
    let (program, diagnostics) = fpas_parser::parse(source);
    assert!(diagnostics.is_empty(), "{diagnostics:?}");
    let executable = fpas_compiler::compile(&program).expect("compile Boolean watch fixture");
    let mut server =
        JsonlServer::new(PreparedDebugTarget::new(executable, Vec::new())).expect("server");
    let _ = server.handle_line(&request(1, "initialize", json!({"version":2})));
    let _ = server.handle_line(&request(2, "launch", json!({"stop_on_entry":true})));

    for (index, (expression, expected)) in [
        ("false and Denied()", "false"),
        ("true or Denied()", "true"),
        ("Guarded()", "false"),
        ("Identity(false) and Identity(true)", "false"),
        ("Identity(true) and Identity(false)", "false"),
        ("Identity(false) or Identity(true)", "true"),
        ("Identity(true) or Identity(false)", "true"),
        ("false and (1 div 0 > 0)", "false"),
        ("true or (1 div 0 > 0)", "true"),
        ("false and ([1][5] = 1)", "false"),
        ("true or ([1][5] = 1)", "true"),
        ("false and (try Error('skipped'))", "false"),
        ("true or (try None)", "true"),
        ("true and (try Some(false))", "false"),
        ("false or (try Ok(true))", "true"),
        ("(false and Denied()) or Identity(true)", "true"),
        ("false and true and Denied()", "false"),
        ("true or false or Denied()", "true"),
        ("BitAnd(0, 7)", "0"),
        ("BitOr(-1, 7)", "-1"),
    ]
    .into_iter()
    .enumerate()
    {
        let records = server.handle_line(&request(
            10 + index as u64,
            "evaluate",
            json!({"expression":expression}),
        ));
        assert_eq!(
            records[0]["body"]["result"], expected,
            "{expression}: {records:?}"
        );
    }

    for (index, expression) in [
        "true and (1 div 0 > 0)",
        "false or (1 div 0 > 0)",
        "false xor (1 div 0 > 0)",
        "true xor (1 div 0 > 0)",
        "BitAnd(0, 1 div 0)",
        "BitOr(-1, 1 div 0)",
        "true and (try Error('needed'))",
        "false or (try None)",
    ]
    .into_iter()
    .enumerate()
    {
        let records = server.handle_line(&request(
            100 + index as u64,
            "evaluate",
            json!({"expression":expression}),
        ));
        assert_eq!(
            records[0]["error"]["code"], "evaluation_domain",
            "{expression}: {records:?}"
        );
    }
    for (index, expression) in [
        "true and Denied()",
        "false or Denied()",
        "false xor Denied()",
    ]
    .into_iter()
    .enumerate()
    {
        let records = server.handle_line(&request(
            200 + index as u64,
            "evaluate",
            json!({"expression":expression}),
        ));
        assert_eq!(
            records[0]["error"]["code"], "call_effect_forbidden",
            "{expression}: {records:?}"
        );
    }
}
