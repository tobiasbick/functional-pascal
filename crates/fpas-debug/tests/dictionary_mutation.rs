//! JSONL dictionary structure mutation, atomicity, and continuation coverage.

#![allow(
    clippy::expect_used,
    clippy::panic,
    reason = "protocol tests keep fixture failures local"
)]

use fpas_debug::{PreparedDebugTarget, jsonl::JsonlServer};
use serde_json::{Value, json};

const SOURCE: &str = r#"program DictionaryMutation;

uses Std.Console;

type
  Row = record
    Scores: dict of string to integer;
  end;

mutable var
  GlobalScores: dict of string to integer := ['Root': 7];

begin
  mutable var Scores: dict of string to integer := [
    'Ada': 1,
    'Grace': 2,
    'Linus': 3
  ];
  var FixedScores: dict of string to integer := ['Fixed': 5];
  mutable var NestedValues: dict of string to array of integer := [:];
  mutable var Rows: array of Row := [
    record
      Scores := ['Left': 10];
    end,
    record
      Scores := ['Right': 20];
    end
  ];
  WriteLn(Scores['Hopper']);
  WriteLn(Scores['Bob']);
  WriteLn(Rows[1].Scores['Nested']);
  WriteLn(GlobalScores['Zed']);
  WriteLn(FixedScores['Fixed'])
end.
"#;

fn server() -> JsonlServer {
    let (program, diagnostics) = fpas_parser::parse(SOURCE);
    assert!(diagnostics.is_empty(), "parse diagnostics: {diagnostics:?}");
    let executable = fpas_compiler::compile(&program).expect("compile dictionary fixture");
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
    for _ in 0..32 {
        let current = frame(server, id);
        let ready = send(
            server,
            id,
            "evaluate",
            json!({"frame_id":current,"expression":"Rows"}),
        );
        if ready[0]["success"] == true {
            return current;
        }
        let step = send(server, id, "step_into", json!({}));
        assert_eq!(step[0]["success"], true, "{step:?}");
        let _ = server.wait();
    }
    panic!("dictionary fixture locals never became initialized")
}

#[test]
fn jsonl_dictionary_mutations_commit_atomically_and_continue() {
    let mut server = server();
    let mut id = 0;
    let initialized = send(&mut server, &mut id, "initialize", json!({"version":2}));
    for capability in [
        "dictionary_insert",
        "dictionary_remove",
        "dictionary_replace_key",
    ] {
        assert_eq!(
            initialized[0]["body"]["capabilities"][capability], true,
            "{capability}"
        );
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
        "dictionary.insert",
        json!({"frame_id":initial_frame,"target":"Scores","key":"'Bob'","expression":"2 + 2"}),
    );
    assert_eq!(inserted[0]["success"], true, "{inserted:?}");
    assert_eq!(inserted[0]["body"]["result"], "{4 entries}");
    assert_eq!(inserted[0]["body"]["named_variables"], 8);
    assert!(inserted[0]["body"]["variables_reference"].as_u64() > Some(0));

    let stale = send(
        &mut server,
        &mut id,
        "dictionary.remove",
        json!({"frame_id":initial_frame,"target":"Scores","key":"'Ada'"}),
    );
    assert_eq!(stale[0]["error"]["code"], "unknown_frame");

    let current = frame(&mut server, &mut id);
    let removed = send(
        &mut server,
        &mut id,
        "dictionary.remove",
        json!({"frame_id":current,"target":"Scores","key":"'Ada'"}),
    );
    assert_eq!(removed[0]["body"]["result"], "{3 entries}", "{removed:?}");
    assert_eq!(removed[0]["body"]["removed"], "1");

    let current = frame(&mut server, &mut id);
    let replaced = send(
        &mut server,
        &mut id,
        "dictionary.replace_key",
        json!({"frame_id":current,"target":"Scores","key":"'Grace'","new_key":"'Hopper'"}),
    );
    assert_eq!(replaced[0]["body"]["old_key"], "'Grace'", "{replaced:?}");
    assert_eq!(replaced[0]["body"]["new_key"], "'Hopper'");

    let current = frame(&mut server, &mut id);
    let nested = send(
        &mut server,
        &mut id,
        "dictionary.insert",
        json!({"frame_id":current,"target":"Rows[1].Scores","key":"'Nested'","expression":"9"}),
    );
    assert_eq!(nested[0]["body"]["result"], "{2 entries}", "{nested:?}");

    let global = send(
        &mut server,
        &mut id,
        "dictionary.insert",
        json!({"target":"GlobalScores","key":"'Zed'","expression":"8"}),
    );
    assert_eq!(global[0]["body"]["result"], "{2 entries}", "{global:?}");

    let current = frame(&mut server, &mut id);
    let aggregate = send(
        &mut server,
        &mut id,
        "dictionary.insert",
        json!({"frame_id":current,"target":"NestedValues","key":"'Pair'","expression":"[4, 5]"}),
    );
    assert_eq!(
        aggregate[0]["body"]["result"], "{1 entries}",
        "{aggregate:?}"
    );

    let current = frame(&mut server, &mut id);
    for (command, arguments, code) in [
        (
            "dictionary.insert",
            json!({"frame_id":current,"target":"Scores","key":"'Linus'","expression":"99"}),
            "dictionary_key_exists",
        ),
        (
            "dictionary.remove",
            json!({"frame_id":current,"target":"Scores","key":"'Missing'"}),
            "dictionary_key_missing",
        ),
        (
            "dictionary.replace_key",
            json!({"frame_id":current,"target":"Scores","key":"'Linus'","new_key":"'Linus'"}),
            "dictionary_key_unchanged",
        ),
        (
            "dictionary.replace_key",
            json!({"frame_id":current,"target":"Scores","key":"'Linus'","new_key":"'Bob'"}),
            "dictionary_key_exists",
        ),
        (
            "dictionary.insert",
            json!({"frame_id":current,"target":"Scores","key":"1","expression":"9"}),
            "variable_value_type",
        ),
        (
            "dictionary.insert",
            json!({"frame_id":current,"target":"Scores","key":"'Wrong'","expression":"'text'"}),
            "variable_value_type",
        ),
        (
            "dictionary.insert",
            json!({"frame_id":current,"target":"Scores['Linus']","key":"'Wrong'","expression":"1"}),
            "variable_path_unsupported",
        ),
        (
            "dictionary.insert",
            json!({"frame_id":current,"target":"FixedScores","key":"'Wrong'","expression":"1"}),
            "variable_not_mutable",
        ),
    ] {
        let failed = send(&mut server, &mut id, command, arguments);
        assert_eq!(failed[0]["error"]["code"], code, "{failed:?}");
        let preserved = send(
            &mut server,
            &mut id,
            "evaluate",
            json!({"frame_id":current,"expression":"Scores['Linus']"}),
        );
        assert_eq!(preserved[0]["body"]["result"], "3", "{preserved:?}");
    }

    let _ = send(&mut server, &mut id, "continue", json!({}));
    let terminated = server.wait();
    let output = terminated
        .iter()
        .filter(|record| record["event"] == "output")
        .filter_map(|record| record["body"]["text"].as_str())
        .collect::<String>();
    assert_eq!(output, "2\n4\n9\n8\n5\n");
}

#[test]
fn jsonl_dictionary_requests_validate_lifecycle_arguments_and_parse_errors() {
    let mut created = server();
    let invalid = created.handle_line(&request(
        1,
        "dictionary.insert",
        json!({"target":"GlobalScores","key":"'X'","expression":"1"}),
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
            "dictionary.insert",
            json!({"key":"'X'","expression":"1"}),
            "invalid_request",
        ),
        (
            "dictionary.insert",
            json!({"target":"GlobalScores","expression":"1"}),
            "invalid_request",
        ),
        (
            "dictionary.remove",
            json!({"target":"GlobalScores","key":1}),
            "invalid_request",
        ),
        (
            "dictionary.replace_key",
            json!({"target":"GlobalScores","key":"'Root'"}),
            "invalid_request",
        ),
        (
            "dictionary.insert",
            json!({"target":"GlobalScores[","key":"'X'","expression":"1"}),
            "expression_target_parse",
        ),
        (
            "dictionary.insert",
            json!({"target":"GlobalScores","key":"(","expression":"1"}),
            "expression_parse",
        ),
        (
            "dictionary.insert",
            json!({"target":"GlobalScores","key":"'X'","expression":"1","frame_id":"top"}),
            "invalid_request",
        ),
    ] {
        let failed = send(&mut server, &mut id, command, arguments);
        assert_eq!(failed[0]["error"]["code"], code, "{failed:?}");
        assert!(
            failed[0]["error"]["help"]
                .as_str()
                .is_some_and(|help| !help.is_empty())
        );
    }
}
