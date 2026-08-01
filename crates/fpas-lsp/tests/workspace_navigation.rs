#![allow(
    clippy::expect_used,
    reason = "protocol fixtures use explicit assertions for hard-coded LSP transcripts"
)]

mod support;

use serde_json::{Value, json};

use support::{TempDirectory, exit, initialize_with_root, initialized, response, run, shutdown};

#[test]
fn phase09_navigation_capabilities_and_utf16_results_are_exposed() {
    let temp = TempDirectory::new("workspace-navigation");
    temp.write(
        "navigation.fpasprj",
        "[project]\nname = \"navigation\"\nkind = \"program\"\nmain = \"src/main.fpas\"\n\n[sources]\ninclude = [\"src/**/*.fpas\"]\n",
    );
    temp.write(
        "src/types.fpas",
        "unit Demo.Types;\n\npublic type Point = record public X: integer; end;\npublic function Create(): Point; begin return record X := 1; end end;\n",
    );
    temp.write(
        "src/other.fpas",
        "unit Demo.Other;\n\npublic function Create(): integer; begin return 2 end;\n",
    );
    let source = "program Nav;\n\nuses Demo.Types, Demo.Other;\n\nmutable var Value: integer := 1;\n\nbegin\n  var Music: string := '𝄞'; Value := Value + 1;\n  var Item: Point := Demo.Types.Create()\nend.\n";
    temp.write("src/main.fpas", source);
    let root_uri = tower_lsp_server::ls_types::Uri::from_file_path(temp.path())
        .expect("root URI")
        .to_string();
    let uri = temp.uri("src/main.fpas");
    let write = source.find("Value := Value").expect("write occurrence");
    let type_query = source.find("Item: Point").expect("typed variable");
    let read = source.find("Value + 1").expect("read occurrence");
    let transcript = run(&[
        initialize_with_root(1, Some(&root_uri)),
        initialized(),
        open(&uri, source),
        workspace_symbols(2, "Create"),
        text_position_request(
            3,
            "textDocument/documentHighlight",
            &uri,
            position(source, write),
        ),
        text_position_request(
            4,
            "textDocument/typeDefinition",
            &uri,
            position(source, type_query),
        ),
        selection_ranges(5, &uri, vec![position(source, read)]),
        shutdown(6),
        exit(),
    ]);

    assert!(
        transcript.output.status.success(),
        "{}",
        String::from_utf8_lossy(&transcript.output.stderr)
    );
    let capabilities = &response(&transcript.messages, 1)["result"]["capabilities"];
    assert_eq!(capabilities["workspaceSymbolProvider"], json!(true));
    assert_eq!(capabilities["documentHighlightProvider"], json!(true));
    assert_eq!(capabilities["typeDefinitionProvider"], json!(true));
    assert_eq!(capabilities["selectionRangeProvider"], json!(true));

    let symbols = response(&transcript.messages, 2)["result"]
        .as_array()
        .expect("workspace symbols");
    assert_eq!(symbols.len(), 2, "{symbols:?}");
    assert_ne!(symbols[0]["containerName"], symbols[1]["containerName"]);

    let highlights = response(&transcript.messages, 3)["result"]
        .as_array()
        .expect("document highlights");
    assert_eq!(highlights.len(), 3, "{highlights:?}");
    assert!(highlights.iter().any(|highlight| highlight["kind"] == 3));

    let definitions = response(&transcript.messages, 4)["result"]
        .as_array()
        .expect("type definitions");
    assert_eq!(definitions.len(), 1, "{definitions:?}");
    assert!(
        definitions[0]["uri"]
            .as_str()
            .is_some_and(|value| value.ends_with("/types.fpas")),
        "{definitions:?}"
    );

    let selection = &response(&transcript.messages, 5)["result"][0];
    assert_eq!(selection["range"]["start"], position(source, read));
    assert_eq!(
        selection["range"]["end"],
        position(source, read + "Value".len())
    );
    assert!(selection["parent"].is_object(), "{selection:?}");
}

fn open(uri: &str, source: &str) -> Value {
    json!({
        "jsonrpc": "2.0",
        "method": "textDocument/didOpen",
        "params": {"textDocument": {
            "uri": uri,
            "languageId": "fpas",
            "version": 1,
            "text": source
        }}
    })
}

fn workspace_symbols(id: i32, query: &str) -> Value {
    json!({
        "jsonrpc": "2.0",
        "id": id,
        "method": "workspace/symbol",
        "params": {"query": query}
    })
}

fn text_position_request(id: i32, method: &str, uri: &str, position: Value) -> Value {
    json!({
        "jsonrpc": "2.0",
        "id": id,
        "method": method,
        "params": {"textDocument": {"uri": uri}, "position": position}
    })
}

fn selection_ranges(id: i32, uri: &str, positions: Vec<Value>) -> Value {
    json!({
        "jsonrpc": "2.0",
        "id": id,
        "method": "textDocument/selectionRange",
        "params": {"textDocument": {"uri": uri}, "positions": positions}
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
