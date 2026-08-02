#![allow(
    clippy::expect_used,
    clippy::panic,
    reason = "protocol fixtures use explicit assertions for hard-coded LSP transcripts"
)]

mod support;

use serde_json::{Value, json};

use support::{
    TempDirectory, TranscriptStep, exit, initialize_with_root, initialize_without_document_changes,
    initialized, response, run, run_script, shutdown,
};

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

    let document_edit = &response(&transcript.messages, 4)["result"]["documentChanges"][0];
    assert_eq!(document_edit["textDocument"]["uri"], uri);
    assert_eq!(document_edit["textDocument"]["version"], 1);
    let changes = document_edit["edits"].as_array().expect("rename edits");
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

#[test]
fn rename_versions_open_documents_and_leaves_disk_documents_unversioned() {
    let temp = TempDirectory::new("cross-document-rename");
    temp.write(
        "demo.fpasprj",
        "[project]\nname = \"demo\"\nkind = \"program\"\nmain = \"src/main.fpas\"\n\n[sources]\ninclude = [\"src/**/*.fpas\"]\n",
    );
    temp.write(
        "src/core.fpas",
        "unit Demo.Core;\n\npublic function Answer(): integer;\nbegin return 42 end;\n",
    );
    let source = "program Demo;\n\nuses Demo.Core;\n\nbegin var Value: integer := Answer() end.\n";
    temp.write("src/main.fpas", source);
    let root_uri = temp.uri(".");
    let main_uri = temp.uri("src/main.fpas");
    let core_uri = temp.uri("src/core.fpas");
    let answer = source.find("Answer").expect("Answer use");
    let transcript = run(&[
        initialize_with_root(1, Some(&root_uri)),
        initialized(),
        open(&main_uri, 7, source),
        rename(2, &main_uri, position(source, answer), "ResultValue"),
        shutdown(3),
        exit(),
    ]);

    assert!(transcript.output.status.success());
    let changes = response(&transcript.messages, 2)["result"]["documentChanges"]
        .as_array()
        .expect("versioned document changes");
    assert_eq!(changes.len(), 2, "{changes:#?}");
    let main = changes
        .iter()
        .find(|change| change["textDocument"]["uri"] == main_uri)
        .expect("open main edit");
    assert_eq!(main["textDocument"]["version"], 7);
    let core = changes
        .iter()
        .find(|change| change["textDocument"]["uri"] == core_uri)
        .expect("disk core edit");
    assert_eq!(core["textDocument"]["version"], Value::Null);
}

#[test]
fn clients_without_document_changes_receive_no_unversioned_edits() {
    let temp = TempDirectory::new("rename-capability");
    let source = "program Capability;\n\nvar Value: integer;\n\nbegin Value := 1 end.\n";
    temp.write("main.fpas", source);
    let root_uri = temp.uri(".");
    let uri = temp.uri("main.fpas");
    let value = source.find("Value").expect("Value declaration");
    let transcript = run(&[
        initialize_without_document_changes(1, Some(&root_uri)),
        initialized(),
        open(&uri, 1, source),
        rename(2, &uri, position(source, value), "Renamed"),
        code_actions(3, &uri),
        shutdown(4),
        exit(),
    ]);

    assert!(transcript.output.status.success());
    assert_eq!(response(&transcript.messages, 2)["result"], Value::Null);
    assert_eq!(response(&transcript.messages, 3)["result"], json!([]));
}

#[test]
fn watched_manifest_change_refreshes_reverse_consumers_without_restart() {
    let temp = TempDirectory::new("watched-project-index");
    let library = temp.write(
        "core/src/core.fpas",
        "unit Demo.Core;\n\npublic function Answer(): integer;\nbegin return 42 end;\n",
    );
    temp.write(
        "core/core.fpasprj",
        "[project]\nname = \"core\"\nkind = \"library\"\n\n[sources]\ninclude = [\"src/**/*.fpas\"]\n",
    );
    let app_manifest = temp.write(
        "app/app.fpasprj",
        "[project]\nname = \"app\"\nkind = \"program\"\nmain = \"src/main.fpas\"\n\n[sources]\ninclude = [\"src/**/*.fpas\"]\n",
    );
    temp.write(
        "app/src/main.fpas",
        "program App;\n\nuses Demo.Core;\n\nbegin var Value: integer := Answer() end.\n",
    );
    let library_uri = temp.uri("core/src/core.fpas");
    let manifest_uri = temp.uri("app/app.fpasprj");
    let root_uri = tower_lsp_server::ls_types::Uri::from_file_path(temp.path())
        .expect("root URI")
        .to_string();
    let source = std::fs::read_to_string(&library).expect("library source");
    let answer = source.find("Answer").expect("Answer declaration");
    let updated_manifest = app_manifest.clone();
    let transcript = run_script(&[
        TranscriptStep::Message(initialize_with_root(1, Some(&root_uri))),
        TranscriptStep::Message(initialized()),
        TranscriptStep::Message(open(&library_uri, 1, &source)),
        TranscriptStep::Message(references(
            2,
            &library_uri,
            position(&source, answer),
            false,
        )),
        TranscriptStep::Action(Box::new(move || {
            std::fs::write(
                &updated_manifest,
                "[project]\nname = \"app\"\nkind = \"program\"\nmain = \"src/main.fpas\"\n\n[dependencies]\nprojects = [\"../core/core.fpasprj\"]\n\n[sources]\ninclude = [\"src/**/*.fpas\"]\n",
            )
            .expect("update app dependency");
        })),
        TranscriptStep::Message(watched_file(&manifest_uri)),
        TranscriptStep::Message(references(
            3,
            &library_uri,
            position(&source, answer),
            false,
        )),
        TranscriptStep::Message(shutdown(4)),
        TranscriptStep::Message(exit()),
    ]);

    assert!(transcript.output.status.success());
    assert_eq!(response(&transcript.messages, 2)["result"], json!([]));
    assert_eq!(
        response(&transcript.messages, 3)["result"]
            .as_array()
            .expect("refreshed references")
            .len(),
        1
    );
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

fn watched_file(uri: &str) -> Value {
    json!({
        "jsonrpc": "2.0",
        "method": "workspace/didChangeWatchedFiles",
        "params": {"changes": [{"uri": uri, "type": 2}]}
    })
}

fn code_actions(id: i32, uri: &str) -> Value {
    json!({
        "jsonrpc": "2.0",
        "id": id,
        "method": "textDocument/codeAction",
        "params": {
            "textDocument": {"uri": uri},
            "range": {
                "start": {"line": 0, "character": 0},
                "end": {"line": 0, "character": 0}
            },
            "context": {"diagnostics": [], "only": ["quickfix"]}
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
