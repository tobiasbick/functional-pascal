#![allow(
    clippy::expect_used,
    clippy::panic,
    reason = "protocol fixtures use explicit source offsets and JSON assertions"
)]

mod support;

use serde_json::{Value, json};
use support::{
    TempDirectory, TranscriptStep, exit, initialize_with_root, initialized, response, run_script,
    shutdown,
};

#[test]
fn completion_resolve_auto_import_and_signature_help_use_utf16_ranges() {
    let temp = TempDirectory::new("intellisense");
    temp.write(
        "demo.fpasprj",
        "[project]\nname = \"demo\"\nkind = \"program\"\nmain = \"src/main.fpas\"\n\n[sources]\ninclude = [\"src/**/*.fpas\"]\n",
    );
    temp.write(
        "src/core.fpas",
        r#"unit Demo.Core;

public type Counter = record
  public Amount: integer;
end;

public function Add(Left: integer; Right: integer): integer;
begin
  return Left + Right
end;
"#,
    );
    let importable = temp.write(
        "src/importable.fpas",
        r#"unit Demo.Importable;

/// Returns a value from the importable unit.
public function UniqueValue(): integer;
begin
  return 42
end;
"#,
    );
    let source = r#"program IntelliSense;

uses Demo.Core;

begin
  var Music: string := '𝄞';
  var CounterValue: Counter := record Amount := 1; end;
  var MemberValue: integer := CounterValue.AmTail;
  var Total: integer := Add(1, Add(2, 3));
  var Imported: integer := UniqueValue
end.
"#;
    temp.write("src/main.fpas", source);
    let root_uri = tower_lsp_server::ls_types::Uri::from_file_path(temp.path())
        .expect("root URI")
        .to_string();
    let main_uri = temp.uri("src/main.fpas");
    let import_uri = tower_lsp_server::ls_types::Uri::from_file_path(&importable)
        .expect("import URI")
        .to_string();
    let member_cursor = source.find("AmTail").expect("member fragment") + 2;
    let import_cursor = source.find("UniqueValue").expect("auto import") + "UniqueValue".len();
    let nested_cursor = source.find("Add(2, 3)").expect("nested call") + "Add(2, ".len();
    let stale_source = "unit Demo.Importable;\n";
    let stale_uri = import_uri.clone();
    let transcript = run_script(&[
        TranscriptStep::Message(initialize_with_root(1, Some(&root_uri))),
        TranscriptStep::Message(initialized()),
        TranscriptStep::Message(open(&main_uri, source)),
        TranscriptStep::Message(completion(2, &main_uri, position(source, member_cursor))),
        TranscriptStep::Message(completion(3, &main_uri, position(source, import_cursor))),
        TranscriptStep::MessageFrom(Box::new(|messages| {
            completion_resolve(4, item(response(messages, 3), "UniqueValue").clone())
        })),
        TranscriptStep::Message(signature_help(
            5,
            &main_uri,
            position(source, nested_cursor),
        )),
        TranscriptStep::MessageFrom(Box::new(|messages| {
            let mut manipulated = item(response(messages, 3), "UniqueValue").clone();
            manipulated["data"]["qualifiedName"] = json!("Demo.Importable.OtherValue");
            completion_resolve(6, manipulated)
        })),
        TranscriptStep::Message(open(&stale_uri, stale_source)),
        TranscriptStep::MessageFrom(Box::new(|messages| {
            completion_resolve(7, item(response(messages, 3), "UniqueValue").clone())
        })),
        TranscriptStep::Message(shutdown(8)),
        TranscriptStep::Message(exit()),
    ]);

    assert!(
        transcript.output.status.success(),
        "{}",
        String::from_utf8_lossy(&transcript.output.stderr)
    );
    let capabilities = &response(&transcript.messages, 1)["result"]["capabilities"];
    assert_eq!(capabilities["completionProvider"]["resolveProvider"], true);
    assert_eq!(
        capabilities["signatureHelpProvider"]["triggerCharacters"],
        json!(["(", ","])
    );

    let member = item(response(&transcript.messages, 2), "Amount");
    assert_eq!(member["kind"], json!(5));
    assert_eq!(member["labelDetails"]["description"], "Demo.Core.Counter");
    let replacement = &member["textEdit"]["range"];
    assert_eq!(
        replacement["start"],
        position(source, source.find("AmTail").expect("replacement start"))
    );
    assert_eq!(
        replacement["end"],
        position(
            source,
            source.find("AmTail").expect("replacement end") + "AmTail".len()
        )
    );

    let imported = item(response(&transcript.messages, 3), "UniqueValue");
    assert_eq!(
        imported["detail"],
        "function UniqueValue(): integer (auto import)"
    );
    assert_eq!(
        imported["additionalTextEdits"][0]["newText"],
        "uses Demo.Core, Demo.Importable;"
    );
    assert_eq!(imported["data"]["uri"], import_uri);
    assert!(imported["data"]["sourceRevision"].is_number());
    assert_eq!(
        imported["data"]["qualifiedName"],
        "Demo.Importable.UniqueValue"
    );
    assert_eq!(
        response(&transcript.messages, 4)["result"]["documentation"]["value"],
        "Returns a value from the importable unit."
    );

    let signature = &response(&transcript.messages, 5)["result"];
    assert_eq!(signature["activeParameter"], 1);
    assert_eq!(signature["signatures"][0]["activeParameter"], 1);
    assert_eq!(
        signature["signatures"][0]["label"],
        "function Add(Left: integer; Right: integer): integer"
    );
    assert_eq!(
        signature["signatures"][0]["parameters"][1]["label"],
        json!([28, 42])
    );
    assert_eq!(
        response(&transcript.messages, 6)["result"]["documentation"],
        Value::Null,
        "manipulated opaque identity must not resolve another declaration"
    );
    assert_eq!(
        response(&transcript.messages, 7)["result"]["documentation"],
        Value::Null,
        "stale completion identities must not resolve against a changed snapshot"
    );
}

fn open(uri: &str, text: &str) -> Value {
    json!({
        "jsonrpc": "2.0",
        "method": "textDocument/didOpen",
        "params": {
            "textDocument": {"uri": uri, "languageId": "fpas", "version": 1, "text": text}
        }
    })
}

fn completion(id: i32, uri: &str, position: Value) -> Value {
    text_position_request(id, "textDocument/completion", uri, position)
}

fn signature_help(id: i32, uri: &str, position: Value) -> Value {
    text_position_request(id, "textDocument/signatureHelp", uri, position)
}

fn completion_resolve(id: i32, item: Value) -> Value {
    json!({"jsonrpc": "2.0", "id": id, "method": "completionItem/resolve", "params": item})
}

fn text_position_request(id: i32, method: &str, uri: &str, position: Value) -> Value {
    json!({
        "jsonrpc": "2.0",
        "id": id,
        "method": method,
        "params": {"textDocument": {"uri": uri}, "position": position}
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

fn item<'a>(response: &'a Value, label: &str) -> &'a Value {
    response["result"]
        .as_array()
        .expect("completion array")
        .iter()
        .find(|item| item["label"] == label)
        .unwrap_or_else(|| panic!("missing completion {label}: {response}"))
}
