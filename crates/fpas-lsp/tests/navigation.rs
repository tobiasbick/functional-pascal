#![allow(
    clippy::expect_used,
    clippy::panic,
    reason = "protocol fixtures use explicit assertions for hard-coded LSP transcripts"
)]

mod support;

use serde_json::{Value, json};

use support::{exit, initialize, initialized, response, run, shutdown};

#[test]
fn navigation_capabilities_and_requests_use_utf16_ranges() {
    let uri = "file:///phase6/navigation.fpas";
    let source = "program Nav;\n\n// Adds one **integer** value.\nfunction Add(Value: integer): integer;\nbegin\n  return Value\nend;\n\nbegin\n  var Music: string := '𝄞';\n  var Total: integer := Add(1)\nend.\n";
    let add_use = source.rfind("Add(1)").expect("function use");
    let string_value = source.find("𝄞").expect("Unicode string");
    let transcript = run(&[
        initialize(1),
        initialized(),
        open(uri, 1, source),
        document_symbols(2, uri),
        hover(3, uri, position(source, add_use)),
        definition(4, uri, position(source, add_use)),
        completion(5, uri, position(source, add_use)),
        hover(6, uri, position(source, string_value)),
        shutdown(7),
        exit(),
    ]);

    assert_success(&transcript);
    let capabilities = &response(&transcript.messages, 1)["result"]["capabilities"];
    assert_eq!(capabilities["documentSymbolProvider"], json!(true));
    assert_eq!(capabilities["hoverProvider"], json!(true));
    assert_eq!(capabilities["definitionProvider"], json!(true));
    assert_eq!(
        capabilities["completionProvider"]["triggerCharacters"],
        json!(["."])
    );

    let roots = response(&transcript.messages, 2)["result"]
        .as_array()
        .expect("document symbol roots");
    assert_eq!(roots.len(), 1);
    assert_eq!(roots[0]["name"], json!("Nav"));
    let children = roots[0]["children"].as_array().expect("root children");
    let add = children
        .iter()
        .find(|symbol| symbol["name"] == "Add")
        .expect("Add symbol");
    assert!(
        add["children"]
            .as_array()
            .expect("routine children")
            .iter()
            .any(|symbol| symbol["name"] == "Value")
    );
    assert!(add["range"]["start"]["line"].as_u64().is_some());
    assert!(add["selectionRange"]["end"]["character"].as_u64().is_some());

    assert!(
        response(&transcript.messages, 3)["result"]["contents"]["value"]
            .as_str()
            .is_some_and(|value| value
                == "```pascal\nfunction Add(Value: integer): integer\n```\n\nAdds one **integer** value.")
    );
    let definitions = response(&transcript.messages, 4)["result"]
        .as_array()
        .expect("definition locations");
    assert_eq!(definitions.len(), 1);
    assert!(
        definitions[0]["uri"]
            .as_str()
            .is_some_and(|value| value.ends_with("/phase6/navigation.fpas"))
    );
    assert_eq!(
        definitions[0]["range"]["start"],
        json!({"line": 3, "character": 9})
    );

    let completions = response(&transcript.messages, 5)["result"]
        .as_array()
        .expect("completion items");
    assert!(
        completions
            .iter()
            .any(|item| item["label"] == "Add" && item["detail"].is_string())
    );
    assert_eq!(response(&transcript.messages, 6)["result"], Value::Null);
}

#[test]
fn partial_source_returns_partial_symbols_and_empty_unknown_navigation() {
    let uri = "file:///phase6/partial.fpas";
    let source = "program Partial;\nbegin\n  Unknown.\nend.\n";
    let unknown = source.find("Unknown").expect("unknown name");
    let transcript = run(&[
        initialize(1),
        initialized(),
        open(uri, 1, source),
        document_symbols(2, uri),
        hover(3, uri, position(source, unknown)),
        definition(4, uri, position(source, unknown)),
        completion(5, uri, position(source, unknown + "Unknown.".len())),
        shutdown(6),
        exit(),
    ]);

    assert_success(&transcript);
    assert_eq!(
        response(&transcript.messages, 2)["result"][0]["name"],
        json!("Partial")
    );
    assert_eq!(response(&transcript.messages, 3)["result"], Value::Null);
    assert_eq!(response(&transcript.messages, 4)["result"], Value::Null);
    assert_eq!(response(&transcript.messages, 5)["result"], json!([]));
}

fn open(uri: &str, version: i32, text: &str) -> Value {
    json!({
        "jsonrpc": "2.0",
        "method": "textDocument/didOpen",
        "params": {
            "textDocument": {
                "uri": uri,
                "languageId": "fpas",
                "version": version,
                "text": text
            }
        }
    })
}

fn document_symbols(id: i32, uri: &str) -> Value {
    json!({
        "jsonrpc": "2.0",
        "id": id,
        "method": "textDocument/documentSymbol",
        "params": {"textDocument": {"uri": uri}}
    })
}

fn hover(id: i32, uri: &str, position: Value) -> Value {
    text_position_request(id, "textDocument/hover", uri, position)
}

fn definition(id: i32, uri: &str, position: Value) -> Value {
    text_position_request(id, "textDocument/definition", uri, position)
}

fn completion(id: i32, uri: &str, position: Value) -> Value {
    text_position_request(id, "textDocument/completion", uri, position)
}

fn text_position_request(id: i32, method: &str, uri: &str, position: Value) -> Value {
    json!({
        "jsonrpc": "2.0",
        "id": id,
        "method": method,
        "params": {
            "textDocument": {"uri": uri},
            "position": position
        }
    })
}

fn position(source: &str, offset: usize) -> Value {
    let before = &source[..offset];
    let line = before.bytes().filter(|byte| *byte == b'\n').count();
    let line_start = before.rfind('\n').map_or(0, |index| index + 1);
    json!({
        "line": line,
        "character": source[line_start..offset].encode_utf16().count()
    })
}

fn assert_success(transcript: &support::Transcript) {
    assert!(
        transcript.output.status.success(),
        "{}",
        String::from_utf8_lossy(&transcript.output.stderr)
    );
}
