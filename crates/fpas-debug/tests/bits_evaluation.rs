//! `Std.Bits` watch calls share runtime bounds and deterministic-call classification.

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
fn bits_watch_calls_preserve_signed_patterns_and_reject_invalid_shift_counts() {
    let (program, errors) = fpas_parser::parse("program BitsWatch; uses Std.Bits; begin end.");
    assert!(errors.is_empty(), "{errors:?}");
    let executable = fpas_compiler::compile(&program).expect("compile bit watch fixture");
    let mut server =
        JsonlServer::new(PreparedDebugTarget::new(executable, Vec::new())).expect("server");
    let _ = server.handle_line(&request(1, "initialize", json!({"version":2})));
    let _ = server.handle_line(&request(2, "launch", json!({"stop_on_entry":true})));

    for (index, (expression, expected)) in [
        ("BitAnd(12, 10)", "8"),
        ("Std.Bits.BitOr(12, 10)", "14"),
        ("BitXor(12, 10)", "6"),
        ("BitNot(0)", "-1"),
        ("ShiftLeft(1, 63)", "-9223372036854775808"),
        ("ShiftLeft(9223372036854775807, 1)", "-2"),
        ("ShiftRight(-8, 1)", "-4"),
        ("ShiftRight(-1, 63)", "-1"),
    ]
    .into_iter()
    .enumerate()
    {
        let result = server.handle_line(&request(
            10 + index as u64,
            "evaluate",
            json!({"expression":expression}),
        ));
        assert_eq!(
            result[0]["body"]["result"], expected,
            "{expression}: {result:?}"
        );
    }
    for (index, expression) in [
        "ShiftLeft(1, -1)",
        "ShiftLeft(1, 64)",
        "ShiftRight(1, -1)",
        "ShiftRight(1, 64)",
    ]
    .into_iter()
    .enumerate()
    {
        let result = server.handle_line(&request(
            100 + index as u64,
            "evaluate",
            json!({"expression":expression}),
        ));
        assert_eq!(
            result[0]["error"]["code"], "call_runtime",
            "{expression}: {result:?}"
        );
        assert!(
            result[0]["error"]["message"]
                .as_str()
                .is_some_and(|message| message.contains("0..63")),
            "{result:?}"
        );
    }
}
