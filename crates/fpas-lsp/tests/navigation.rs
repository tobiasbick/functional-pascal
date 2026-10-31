//! Protocol tests for LSP definition and symbol navigation.

#![allow(
    clippy::expect_used,
    clippy::panic,
    reason = "protocol fixtures use explicit assertions for hard-coded LSP transcripts"
)]

use crate::support;

use serde_json::{Value, json};

use support::{
    TempDirectory, exit, initialize, initialize_with_root, initialized, response, run, shutdown,
};

#[test]
fn local_inference_hover_is_serialized_for_unsaved_declarations_and_uses() {
    let temp = TempDirectory::new("hover-inference");
    let main_uri = temp.uri("main.fpas");
    let uri = main_uri.as_str();
    let source = "program T; begin const Text := '𝄞'; const Values := [1, 2]; discard Values; end.";
    let declaration = source.find("Values :=").expect("declaration");
    let usage = source.find("discard Values").expect("use") + "discard ".len();
    let transcript = run(&[
        initialize_with_root(1, Some(&temp.uri("."))),
        initialized(),
        open(uri, 1, source),
        hover(2, uri, position(source, declaration)),
        hover(3, uri, position(source, usage)),
        shutdown(4),
        exit(),
    ]);
    assert_success(&transcript);
    for id in [2, 3] {
        let column = position(source, declaration)["character"]
            .as_u64()
            .expect("column")
            + 1;
        assert_eq!(
            response(&transcript.messages, id)["result"]["contents"]["value"],
            json!(format!(
                "```fpas\nconst Values: array of integer\n```\n\n[Go to definition](<{uri}#L1,{column}>)"
            ))
        );
    }
    assert_eq!(
        response(&transcript.messages, 3)["result"]["range"]["start"],
        position(source, usage)
    );
}

#[test]
fn navigation_capabilities_and_requests_use_utf16_ranges() {
    let temp = TempDirectory::new("hover-navigation");
    let main_uri = temp.uri("navigation.fpas");
    let uri = main_uri.as_str();
    let source = "program Nav;\n\n// Adds two **integer** values.\nfunction Add(Value: integer; Extra: integer): integer;\nbegin\n  return Value + Extra;\nend function;\n\nbegin\n  const Music: string := '𝄞';\n  const Total: integer := Add(1, 2);\nend.\n";
    let add_use = source.rfind("Add(1, 2)").expect("function use");
    let string_value = source.find("𝄞").expect("Unicode string");
    let mut init = initialize_with_root(1, Some(&temp.uri(".")));
    init["params"]["capabilities"]["textDocument"]["definition"] = json!({"linkSupport": true});
    let transcript = run(&[
        init,
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
                == format!("```fpas\nfunction Add(\n  Value: integer;\n  Extra: integer\n): integer\n```\n\n---\n\nAdds two **integer** values.\n\n[Go to definition](<{uri}#L4,10>)"))
    );
    let definitions = response(&transcript.messages, 4)["result"]
        .as_array()
        .expect("definition locations");
    assert_eq!(definitions.len(), 1);
    assert!(
        definitions[0]["targetUri"]
            .as_str()
            .is_some_and(|value| value == uri)
    );
    assert_eq!(
        definitions[0]["targetSelectionRange"]["start"],
        json!({"line": 3, "character": 9})
    );
    assert_eq!(
        definitions[0]["targetRange"]["start"],
        json!({"line": 3, "character": 0})
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
