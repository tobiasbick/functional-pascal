//! Debugger watches use program precedence and reject ambiguous logical chains.

use fpas_debug::{PreparedDebugTarget, jsonl::JsonlServer};
use serde_json::{Value, json};

fn request(id: u64, command: &str, arguments: Value) -> String {
    json!({"type":"request", "id":id, "command":command, "arguments":arguments}).to_string()
}

#[test]
fn watches_apply_logical_precedence_and_report_mixing_and_comparison_errors() {
    let (program, diagnostics) = fpas_parser::parse("program Watch; begin end.");
    assert!(diagnostics.is_empty());
    let executable = fpas_compiler::compile(&program).expect("watch fixture");
    let mut server =
        JsonlServer::new(PreparedDebugTarget::new(executable, Vec::new())).expect("server");
    let _ = server.handle_line(&request(1, "initialize", json!({"version":2})));
    let _ = server.handle_line(&request(2, "launch", json!({"stop_on_entry":true})));
    for (index, (expression, expected)) in [
        ("not 2 > 0", "false"),
        ("not 2 = 3", "true"),
        ("not 2 in [1, 2]", "false"),
        ("1 < 2 and 2 < 3", "true"),
        ("2 < 1 or 3 > 2", "true"),
        ("1 < 2 xor 2 > 3", "true"),
        ("not false and true", "true"),
        ("not not 1 < 2", "true"),
        ("false and 1 div 0 > 0", "false"),
        ("true or 1 div 0 > 0", "true"),
        ("(true or false) and false", "false"),
        ("true or (false and false)", "true"),
        ("try Some(2) + 1 = 3", "true"),
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
    let mut id = 100;
    for first in ["and", "or", "xor"] {
        for second in ["and", "or", "xor"] {
            if first == second {
                continue;
            }
            let expression = format!("true {first} false {second} true");
            let result =
                server.handle_line(&request(id, "evaluate", json!({"expression":expression})));
            id += 1;
            assert_eq!(result[0]["error"]["code"], "expression_parse", "{result:?}");
            assert!(
                result[0]["error"]["message"]
                    .as_str()
                    .unwrap()
                    .contains("require parentheses"),
                "{result:?}"
            );
        }
    }
    let result = server.handle_line(&request(id, "evaluate", json!({"expression":"1 < 2 < 3"})));
    assert_eq!(result[0]["error"]["code"], "expression_parse", "{result:?}");
}
