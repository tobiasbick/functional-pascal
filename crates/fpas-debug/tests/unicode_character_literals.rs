//! Watch evaluation shares Unicode literal parsing and reports exact invalid-code spans.

use fpas_debug::{PreparedDebugTarget, jsonl::JsonlServer};
use serde_json::json;

#[test]
fn watches_decode_unicode_codes_and_reject_invalid_scalars()
-> Result<(), Box<dyn std::error::Error>> {
    let (program, diagnostics) = fpas_parser::parse("program T; begin end.");
    assert!(diagnostics.is_empty(), "{diagnostics:?}");
    let executable = fpas_compiler::compile(&program).map_err(|errors| format!("{errors:?}"))?;
    let mut server = JsonlServer::new(PreparedDebugTarget::new(executable, Vec::new()))?;
    for (id, command, arguments) in [
        (1, "initialize", json!({"version":2})),
        (2, "launch", json!({"stop_on_entry":true})),
    ] {
        let _ = server.handle_line(
            &json!({"type":"request", "id":id, "command":command, "arguments":arguments})
                .to_string(),
        );
    }

    for (index, (expression, expected)) in [
        ("#128512 = '😀'", "true"),
        ("#8_364 = '€'", "true"),
        ("('x'#8_364#128512)[2] = '😀'", "true"),
        ("'x'#0#128512 = 'x'#0'😀'", "true"),
    ]
    .into_iter()
    .enumerate()
    {
        let records = server.handle_line(
            &json!({"type":"request", "id":10 + index, "command":"evaluate",
                "arguments":{"expression":expression}})
            .to_string(),
        );
        assert_eq!(records[0]["body"]["result"], expected, "{records:?}");
    }

    let records = server.handle_line(
        &json!({"type":"request", "id":20, "command":"evaluate",
            "arguments":{"expression":"'😀'#55296"}})
        .to_string(),
    );
    let error = &records[0]["error"];
    assert_eq!(error["code"], "expression_parse", "{records:?}");
    assert_eq!(error["offset"], 6);
    assert_eq!(error["length"], 6);
    assert!(
        error["help"]
            .as_str()
            .is_some_and(|hint| hint.contains("#8364"))
    );
    Ok(())
}
