#![allow(
    clippy::expect_used,
    clippy::panic,
    reason = "semantic-tool protocol fixtures use explicit JSON assertions"
)]

mod support;

use fpas_diagnostics::codes::SEMA_UNKNOWN_NAME;
use fpas_language_service::{LanguageService, diagnostics_for_document};
use serde_json::{Value, json};
use support::{TempDirectory, exit, initialize_with_root, initialized, response, run, shutdown};

#[test]
fn semantic_tokens_and_quick_fixes_use_utf16_and_reject_stale_diagnostics() {
    let temp = TempDirectory::new("semantic-tools");
    temp.write(
        "actions.fpasprj",
        "[project]\nname = \"actions\"\nkind = \"program\"\nmain = \"src/main.fpas\"\n\n[sources]\ninclude = [\"src/**/*.fpas\"]\n",
    );
    temp.write(
        "src/core.fpas",
        "unit Actions.Core;\n\npublic const ExistingText: string := 'ok';\n\npublic function Existing(): integer;\nbegin\n  return 1\nend;\n",
    );
    temp.write(
        "src/importable.fpas",
        "unit Actions.Importable;\n\npublic function UniqueValue(): integer;\nbegin\n  return 42\nend;\n",
    );
    let source = "program Actions;\n\nuses Actions.Core;\n\nbegin\n  var Music: string := '𝄞' + ExistingText;\n  var Value: integer := UniqueValue()\nend.\n";
    let main_path = temp.write("src/main.fpas", source);
    let root_uri = temp.uri(".");
    let main_uri = temp.uri("src/main.fpas");
    let mut service = LanguageService::load(&temp.path().join("actions.fpasprj"));
    let analysis = service
        .analyze_document(&main_path)
        .expect("fixture analysis");
    let diagnostics = diagnostics_for_document(&analysis);
    let unknown = diagnostics
        .iter()
        .find(|diagnostic| diagnostic.code == SEMA_UNKNOWN_NAME)
        .expect("unknown-name diagnostic");
    let diagnostic = json!({
        "range": range(
            source,
            unknown.span.offset,
            unknown.span.offset + unknown.span.length
        ),
        "severity": 1,
        "code": unknown.code.to_string(),
        "source": "fpas",
        "message": unknown.message
    });
    let changed = source.replace("UniqueValue", "Existing");

    let transcript = run(&[
        initialize_with_root(1, Some(&root_uri)),
        initialized(),
        open(&main_uri, 1, source),
        semantic_tokens(2, &main_uri),
        code_actions(3, &main_uri, &diagnostic),
        change(&main_uri, 2, &changed),
        code_actions(4, &main_uri, &diagnostic),
        shutdown(5),
        exit(),
    ]);

    assert!(
        transcript.output.status.success(),
        "{}",
        String::from_utf8_lossy(&transcript.output.stderr)
    );

    let capabilities = &response(&transcript.messages, 1)["result"]["capabilities"];
    let provider = &capabilities["semanticTokensProvider"];
    assert_eq!(provider["full"], json!(true));
    assert_eq!(provider["range"], Value::Null);
    assert_eq!(
        provider["legend"]["tokenTypes"],
        json!([
            "namespace",
            "type",
            "enum",
            "typeParameter",
            "parameter",
            "variable",
            "field",
            "property",
            "event",
            "enumMember",
            "function",
            "procedure",
            "method",
            "constant"
        ])
    );
    assert_eq!(
        provider["legend"]["tokenModifiers"],
        json!(["declaration", "readonly", "public"])
    );
    assert_eq!(
        capabilities["codeActionProvider"]["codeActionKinds"],
        json!(["quickfix"])
    );
    assert_eq!(
        capabilities["codeActionProvider"]["resolveProvider"],
        json!(false)
    );

    let encoded = response(&transcript.messages, 2)["result"]["data"]
        .as_array()
        .expect("semantic token data");
    let decoded = decode_tokens(encoded);
    assert_token(
        &decoded,
        position(source, source.find("Actions.Core").expect("qualified unit")),
        "Actions".encode_utf16().count(),
        0,
    );
    assert_token(
        &decoded,
        position(
            source,
            source.find("Actions.Core").expect("qualified unit") + "Actions.".len(),
        ),
        "Core".encode_utf16().count(),
        0,
    );
    let existing_text = source.find("ExistingText").expect("constant reference");
    let utf16_position = position(source, existing_text);
    assert!(
        existing_text > utf16_position.1,
        "the non-BMP fixture must distinguish byte and UTF-16 offsets"
    );
    assert_token(
        &decoded,
        utf16_position,
        "ExistingText".encode_utf16().count(),
        13,
    );
    assert!(
        decoded.windows(2).all(|pair| pair[0].0 < pair[1].0
            || (pair[0].0 == pair[1].0 && pair[0].1 + pair[0].2 <= pair[1].1)),
        "semantic tokens overlap or are unstable: {decoded:?}"
    );

    let actions = response(&transcript.messages, 3)["result"]
        .as_array()
        .expect("code action array");
    let [action] = actions.as_slice() else {
        panic!("expected one safe import action: {actions:#?}")
    };
    assert_eq!(action["title"], "Import Actions.Importable");
    assert_eq!(action["kind"], "quickfix");
    assert_eq!(action["isPreferred"], true);
    assert_eq!(action["diagnostics"], json!([diagnostic]));
    let edit = &action["edit"]["changes"][&main_uri][0];
    assert_eq!(edit["newText"], "uses Actions.Core, Actions.Importable;");
    let edited = apply_edit(source, edit);
    let (unit, parse_diagnostics) = fpas_parser::parse_compilation_unit(&edited);
    assert!(parse_diagnostics.is_empty(), "{parse_diagnostics:#?}");
    assert_eq!(fpas_fmt::format_source(&edited, &unit), edited);

    assert_eq!(
        response(&transcript.messages, 4)["result"],
        json!([]),
        "{:?}",
        response(&transcript.messages, 4)
    );
    assert_eq!(response(&transcript.messages, 5)["result"], Value::Null);
}

fn open(uri: &str, version: i32, text: &str) -> Value {
    json!({
        "jsonrpc": "2.0",
        "method": "textDocument/didOpen",
        "params": {
            "textDocument": {"uri": uri, "languageId": "fpas", "version": version, "text": text}
        }
    })
}

fn change(uri: &str, version: i32, text: &str) -> Value {
    json!({
        "jsonrpc": "2.0",
        "method": "textDocument/didChange",
        "params": {
            "textDocument": {"uri": uri, "version": version},
            "contentChanges": [{"text": text}]
        }
    })
}

fn semantic_tokens(id: i32, uri: &str) -> Value {
    json!({
        "jsonrpc": "2.0",
        "id": id,
        "method": "textDocument/semanticTokens/full",
        "params": {"textDocument": {"uri": uri}}
    })
}

fn code_actions(id: i32, uri: &str, diagnostic: &Value) -> Value {
    json!({
        "jsonrpc": "2.0",
        "id": id,
        "method": "textDocument/codeAction",
        "params": {
            "textDocument": {"uri": uri},
            "range": diagnostic["range"],
            "context": {"diagnostics": [diagnostic], "only": ["quickfix"]}
        }
    })
}

fn range(source: &str, start: usize, end: usize) -> Value {
    json!({"start": position_value(source, start), "end": position_value(source, end)})
}

fn position_value(source: &str, offset: usize) -> Value {
    let (line, character) = position(source, offset);
    json!({"line": line, "character": character})
}

fn position(source: &str, offset: usize) -> (usize, usize) {
    let before = &source[..offset];
    let line = before.bytes().filter(|byte| *byte == b'\n').count();
    let line_start = before.rfind('\n').map_or(0, |index| index + 1);
    (line, source[line_start..offset].encode_utf16().count())
}

fn decode_tokens(data: &[Value]) -> Vec<(usize, usize, usize, u64, u64)> {
    assert_eq!(data.len() % 5, 0, "invalid semantic token data: {data:?}");
    let mut line = 0;
    let mut character = 0;
    data.chunks_exact(5)
        .map(|token| {
            let delta_line = token[0].as_u64().expect("delta line") as usize;
            let delta_start = token[1].as_u64().expect("delta start") as usize;
            line += delta_line;
            character = if delta_line == 0 {
                character + delta_start
            } else {
                delta_start
            };
            (
                line,
                character,
                token[2].as_u64().expect("token length") as usize,
                token[3].as_u64().expect("token type"),
                token[4].as_u64().expect("token modifiers"),
            )
        })
        .collect()
}

fn assert_token(
    tokens: &[(usize, usize, usize, u64, u64)],
    position: (usize, usize),
    length: usize,
    token_type: u64,
) {
    assert!(
        tokens.iter().any(|token| {
            (token.0, token.1) == position && token.2 == length && token.3 == token_type
        }),
        "missing token at {position:?} with length {length} and type {token_type}: {tokens:#?}"
    );
}

fn apply_edit(source: &str, edit: &Value) -> String {
    let start = offset(source, &edit["range"]["start"]);
    let end = offset(source, &edit["range"]["end"]);
    let mut edited = source.to_owned();
    edited.replace_range(start..end, edit["newText"].as_str().expect("edit text"));
    edited
}

fn offset(source: &str, position: &Value) -> usize {
    let line = position["line"].as_u64().expect("edit line") as usize;
    let character = position["character"].as_u64().expect("edit character") as usize;
    let line_start = source
        .match_indices('\n')
        .nth(line.saturating_sub(1))
        .map_or(0, |(index, _)| index + 1);
    let line_end = source[line_start..]
        .find('\n')
        .map_or(source.len(), |length| line_start + length);
    let mut utf16 = 0;
    for (relative, value) in source[line_start..line_end].char_indices() {
        if utf16 == character {
            return line_start + relative;
        }
        utf16 += value.len_utf16();
    }
    assert_eq!(utf16, character, "position is not a UTF-16 boundary");
    line_end
}
