//! JSONL array and string structure mutation, atomicity, and continuation coverage.

#![allow(
    clippy::expect_used,
    clippy::panic,
    reason = "protocol tests keep fixture failures local"
)]

use fpas_debug::{PreparedDebugTarget, jsonl::JsonlServer};
use serde_json::{Value, json};

const SOURCE: &str = r#"program JsonlSequenceMutation;

uses Std.Console;

begin
  mutable var Numbers: array of integer := [1, 2, 3];
  var Fixed: array of integer := [8];
  mutable var Text: string := 'A😀B';
  WriteLn(Numbers[0]);
  WriteLn(Numbers[1]);
  WriteLn(Numbers[2]);
  WriteLn(Text);
  WriteLn(Fixed[0])
end.
"#;

fn server() -> JsonlServer {
    let (program, diagnostics) = fpas_parser::parse(SOURCE);
    assert!(diagnostics.is_empty(), "parse diagnostics: {diagnostics:?}");
    let executable = fpas_compiler::compile(&program).expect("compile sequence fixture");
    JsonlServer::new(PreparedDebugTarget::new(executable, Vec::new())).expect("JSONL server")
}

fn request(id: u64, command: &str, arguments: Value) -> String {
    json!({"type":"request","id":id,"command":command,"arguments":arguments}).to_string()
}

fn send(server: &mut JsonlServer, id: &mut u64, command: &str, arguments: Value) -> Vec<Value> {
    *id += 1;
    server.handle_line(&request(*id, command, arguments))
}

fn frame(server: &mut JsonlServer, id: &mut u64) -> u64 {
    send(server, id, "stack", json!({}))[0]["body"]["frames"][0]["frame_id"]
        .as_u64()
        .expect("frame ID")
}

fn stop_with_initialized_locals(server: &mut JsonlServer, id: &mut u64) -> u64 {
    for _ in 0..24 {
        let current = frame(server, id);
        let ready = send(
            server,
            id,
            "evaluate",
            json!({"frame_id":current,"expression":"Text"}),
        );
        if ready[0]["success"] == true {
            return current;
        }
        let _ = send(server, id, "step_into", json!({}));
        let _ = server.wait();
    }
    panic!("sequence fixture locals never became initialized")
}

#[test]
fn jsonl_sequence_mutations_commit_atomically_and_continue() {
    let mut server = server();
    let mut id = 0;
    let initialized = send(&mut server, &mut id, "initialize", json!({"version":2}));
    for capability in ["array_insert", "array_remove", "string_replace_character"] {
        assert_eq!(initialized[0]["body"]["capabilities"][capability], true);
    }
    let _ = send(
        &mut server,
        &mut id,
        "launch",
        json!({"stop_on_entry":true}),
    );
    let initial_frame = stop_with_initialized_locals(&mut server, &mut id);

    let inserted = send(
        &mut server,
        &mut id,
        "array.insert",
        json!({"frame_id":initial_frame,"target":"Numbers","index":"1","expression":"9"}),
    );
    assert_eq!(inserted[0]["body"]["result"], "[4 items]", "{inserted:?}");
    assert_eq!(inserted[0]["body"]["index"], 1);
    assert_eq!(inserted[0]["body"]["indexed_variables"], 4);

    let current = frame(&mut server, &mut id);
    let removed = send(
        &mut server,
        &mut id,
        "array.remove",
        json!({"frame_id":current,"target":"Numbers","index":"2"}),
    );
    assert_eq!(removed[0]["body"]["removed"], "2", "{removed:?}");
    assert_eq!(removed[0]["body"]["result"], "[3 items]");

    let current = frame(&mut server, &mut id);
    let replaced = send(
        &mut server,
        &mut id,
        "string.replace_character",
        json!({"frame_id":current,"target":"Text","index":"1","expression":"'é'"}),
    );
    assert_eq!(replaced[0]["body"]["result"], "'AéB'", "{replaced:?}");
    assert_eq!(replaced[0]["body"]["old_character"], "'😀'");
    assert_eq!(replaced[0]["body"]["new_character"], "'é'");

    let current = frame(&mut server, &mut id);
    for (command, arguments, code) in [
        (
            "array.insert",
            json!({"frame_id":current,"target":"Numbers","index":"4","expression":"7"}),
            "sequence_index_out_of_bounds",
        ),
        (
            "array.remove",
            json!({"frame_id":current,"target":"Numbers","index":"-1"}),
            "sequence_index_out_of_bounds",
        ),
        (
            "array.insert",
            json!({"frame_id":current,"target":"Numbers","index":"0","expression":"'wrong'"}),
            "variable_value_type",
        ),
        (
            "string.replace_character",
            json!({"frame_id":current,"target":"Text","index":"0","expression":"''"}),
            "string_character_required",
        ),
        (
            "string.replace_character",
            json!({"frame_id":current,"target":"Text","index":"0","expression":"'A'"}),
            "string_character_unchanged",
        ),
        (
            "array.remove",
            json!({"frame_id":current,"target":"Fixed","index":"0"}),
            "variable_not_mutable",
        ),
    ] {
        let failed = send(&mut server, &mut id, command, arguments);
        assert_eq!(failed[0]["error"]["code"], code, "{failed:?}");
        let preserved = send(
            &mut server,
            &mut id,
            "evaluate",
            json!({"frame_id":current,"expression":"Text"}),
        );
        assert_eq!(preserved[0]["body"]["result"], "'AéB'");
    }

    let _ = send(&mut server, &mut id, "continue", json!({}));
    let output = server
        .wait()
        .iter()
        .filter(|record| record["event"] == "output")
        .filter_map(|record| record["body"]["text"].as_str())
        .collect::<String>();
    assert_eq!(output, "1\n9\n3\nAéB\n8\n");
}

#[test]
fn jsonl_sequence_requests_validate_lifecycle_and_arguments() {
    let mut created = server();
    let invalid = created.handle_line(&request(
        1,
        "array.insert",
        json!({"target":"Numbers","index":"0","expression":"1"}),
    ));
    assert_eq!(invalid[0]["error"]["code"], "invalid_state");

    let mut server = server();
    let mut id = 0;
    let _ = send(&mut server, &mut id, "initialize", json!({}));
    let _ = send(
        &mut server,
        &mut id,
        "launch",
        json!({"stop_on_entry":true}),
    );
    for (command, arguments, code) in [
        (
            "array.insert",
            json!({"index":"0","expression":"1"}),
            "invalid_request",
        ),
        (
            "array.remove",
            json!({"target":"Numbers"}),
            "invalid_request",
        ),
        (
            "string.replace_character",
            json!({"target":"Text","index":"0"}),
            "invalid_request",
        ),
        (
            "array.insert",
            json!({"target":"Numbers[","index":"0","expression":"1"}),
            "expression_target_parse",
        ),
        (
            "array.remove",
            json!({"target":"Numbers","index":"("}),
            "expression_parse",
        ),
    ] {
        let failed = send(&mut server, &mut id, command, arguments);
        assert_eq!(failed[0]["error"]["code"], code, "{failed:?}");
    }
}
