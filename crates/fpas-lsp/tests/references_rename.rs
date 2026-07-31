#![allow(
    clippy::expect_used,
    clippy::panic,
    reason = "protocol fixtures use explicit assertions for hard-coded LSP transcripts"
)]

mod support;

use serde_json::{Value, json};

use support::{TempDirectory, exit, initialize_with_root, initialized, response, run, shutdown};

#[test]
fn references_and_rename_use_utf16_ranges_and_workspace_edits() {
    let temp = TempDirectory::new("references-rename");
    let uri = temp.uri("navigation.fpas");
    let root_uri = tower_lsp_server::ls_types::Uri::from_file_path(temp.path())
        .expect("root URI")
        .to_string();
    let source = "program Nav;\n\nfunction Add(Value: integer): integer;\nbegin\n  return Value\nend;\n\nbegin\n  var Music: string := '𝄞';\n  var Total: integer := Add(1)\nend.\n";
    temp.write("navigation.fpas", source);
    let add_use = source.rfind("Add(1)").expect("function use");
    let transcript = run(&[
        initialize_with_root(1, Some(&root_uri)),
        initialized(),
        open(&uri, 1, source),
        references(2, &uri, position(source, add_use), true),
        prepare_rename(3, &uri, position(source, add_use)),
        rename(4, &uri, position(source, add_use), "Sum"),
        rename(5, &uri, position(source, add_use), "begin"),
        shutdown(6),
        exit(),
    ]);

    assert!(
        transcript.output.status.success(),
        "{}",
        String::from_utf8_lossy(&transcript.output.stderr)
    );
    let capabilities = &response(&transcript.messages, 1)["result"]["capabilities"];
    assert_eq!(capabilities["referencesProvider"], json!(true));
    assert_eq!(
        capabilities["renameProvider"]["prepareProvider"],
        json!(true)
    );

    let references = response(&transcript.messages, 2)["result"]
        .as_array()
        .expect("reference locations");
    assert_eq!(references.len(), 2, "{references:?}");
    assert_eq!(
        response(&transcript.messages, 3)["result"]["placeholder"],
        json!("Add")
    );

    let changes = response(&transcript.messages, 4)["result"]["changes"][&uri]
        .as_array()
        .expect("rename edits");
    assert_eq!(changes.len(), 2, "{changes:?}");
    assert!(changes.iter().all(|edit| edit["newText"] == "Sum"));
    let invalid = response(&transcript.messages, 5);
    assert_eq!(invalid["error"]["code"], json!(-32602));
    assert!(
        invalid["error"]["message"]
            .as_str()
            .is_some_and(|message| message.contains("valid rename target"))
    );
}

#[test]
fn references_can_exclude_declarations_and_prepare_rejects_unit_names() {
    let temp = TempDirectory::new("references-options");
    let uri = temp.uri("unit.fpas");
    let root_uri = tower_lsp_server::ls_types::Uri::from_file_path(temp.path())
        .expect("root URI")
        .to_string();
    let source = "unit Demo.UnitName;\n\npublic function Value(): integer;\nbegin return 1 end;\n";
    temp.write("unit.fpas", source);
    let value = source.find("Value").expect("function declaration");
    let unit_name = source.find("UnitName").expect("unit name");
    let transcript = run(&[
        initialize_with_root(1, Some(&root_uri)),
        initialized(),
        open(&uri, 1, source),
        references(2, &uri, position(source, value), false),
        prepare_rename(3, &uri, position(source, unit_name)),
        shutdown(4),
        exit(),
    ]);

    assert!(transcript.output.status.success());
    assert_eq!(response(&transcript.messages, 2)["result"], json!([]));
    assert_eq!(response(&transcript.messages, 3)["result"], Value::Null);
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

fn references(id: i32, uri: &str, position: Value, include_declaration: bool) -> Value {
    json!({
        "jsonrpc": "2.0",
        "id": id,
        "method": "textDocument/references",
        "params": {
            "textDocument": {"uri": uri},
            "position": position,
            "context": {"includeDeclaration": include_declaration}
        }
    })
}

fn prepare_rename(id: i32, uri: &str, position: Value) -> Value {
    text_position_request(id, "textDocument/prepareRename", uri, position, None)
}

fn rename(id: i32, uri: &str, position: Value, new_name: &str) -> Value {
    text_position_request(id, "textDocument/rename", uri, position, Some(new_name))
}

fn text_position_request(
    id: i32,
    method: &str,
    uri: &str,
    position: Value,
    new_name: Option<&str>,
) -> Value {
    let mut params = json!({
        "textDocument": {"uri": uri},
        "position": position
    });
    if let Some(new_name) = new_name {
        params["newName"] = json!(new_name);
    }
    json!({
        "jsonrpc": "2.0",
        "id": id,
        "method": method,
        "params": params
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
