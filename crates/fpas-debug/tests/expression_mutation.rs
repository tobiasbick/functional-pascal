//! JSONL textual-mutation success, atomicity, target-domain, and lifecycle coverage.

#![allow(
    clippy::expect_used,
    clippy::panic,
    reason = "protocol tests keep fixture failures local"
)]

use fpas_debug::{PreparedDebugTarget, jsonl::JsonlServer};
use serde_json::{Value, json};

const SOURCE: &str = r#"program JsonlExpressionMutation;

uses Std.Console;

type
  Box = record
    Value: integer;
    Other: integer;
  end;
  Container = record
    Items: array of Box;
  end;

mutable var
  GlobalValue: integer := 5;

function ChooseIndex(): integer;
begin
  return 1
end;

begin
  mutable var Selected: integer := 0;
  mutable var Counter: integer := 1;
  var Fixed: integer := 2;
  mutable var State: Container := record
    Items := [
      record
        Value := 10;
        Other := 11;
      end,
      record
        Value := 20;
        Other := 21;
      end
    ];
  end;
  mutable var Scores: dict of string to integer := ['blue': 30];
  mutable var Text: string := 'abc';
  Counter := Counter + Fixed;
  WriteLn(GlobalValue);
  WriteLn(Counter);
  WriteLn(State.Items[1].Value);
  WriteLn(Scores['blue']);
  WriteLn(Text)
end.
"#;

fn new_server() -> JsonlServer {
    let (program, diagnostics) = fpas_parser::parse(SOURCE);
    assert!(diagnostics.is_empty(), "parse diagnostics: {diagnostics:?}");
    let executable = fpas_compiler::compile(&program).expect("compile expression mutation fixture");
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
            json!({"frame_id":current,"expression":"Text"}),
        );
        if ready[0]["success"] == true {
            return current;
        }
        let step = send(server, id, "step_into", json!({}));
        assert_eq!(step[0]["success"], true, "{step:?}");
        let _ = server.wait();
    }
    panic!("fixture locals never became initialized")
}

#[test]
fn jsonl_expression_set_supports_bounded_targets_and_atomic_failures() {
    let mut server = new_server();
    let mut id = 0;
    let initialized = send(&mut server, &mut id, "initialize", json!({"version":2}));
    assert_eq!(
        initialized[0]["body"]["capabilities"]["set_expression"],
        true
    );
    let _ = send(
        &mut server,
        &mut id,
        "launch",
        json!({"stop_on_entry":true}),
    );
    let initial_frame = stop_with_initialized_locals(&mut server, &mut id);

    let local = send(
        &mut server,
        &mut id,
        "expression.set",
        json!({"frame_id":initial_frame,"target":"Counter","expression":"Counter + 40"}),
    );
    assert_eq!(local[0]["body"]["result"], "41", "{local:?}");
    assert_eq!(local[0]["body"]["type_name"], "integer");
    assert_eq!(local[0]["body"]["variables_reference"], 0);

    let stale = send(
        &mut server,
        &mut id,
        "expression.set",
        json!({"frame_id":initial_frame,"target":"Counter","expression":"1"}),
    );
    assert_eq!(stale[0]["error"]["code"], "unknown_frame");

    let current = frame(&mut server, &mut id);
    let nested = send(
        &mut server,
        &mut id,
        "expression.set",
        json!({
            "frame_id":current,
            "target":"sTaTe.iTeMs[ChooseIndex()].vAlUe",
            "expression":"Counter + 1"
        }),
    );
    assert_eq!(nested[0]["body"]["result"], "42", "{nested:?}");

    let current = frame(&mut server, &mut id);
    let dictionary = send(
        &mut server,
        &mut id,
        "expression.set",
        json!({
            "frame_id":current,
            "target":"Scores['blue']",
            "expression":"State.Items[1].Value + 1"
        }),
    );
    assert_eq!(dictionary[0]["body"]["result"], "43", "{dictionary:?}");

    let current = frame(&mut server, &mut id);
    let aggregate = send(
        &mut server,
        &mut id,
        "expression.set",
        json!({"frame_id":current,"target":"State","expression":"State"}),
    );
    assert_eq!(aggregate[0]["success"], true, "{aggregate:?}");
    assert_ne!(aggregate[0]["body"]["variables_reference"], 0);
    assert_eq!(aggregate[0]["body"]["named_variables"], 1);

    let current = frame(&mut server, &mut id);
    for (target, expression, code) in [
        ("Fixed", "3", "variable_not_mutable"),
        ("Counter", "'wrong'", "variable_value_type"),
        ("State.Missing", "1", "variable_target_unknown"),
        ("State.Items[99].Value", "1", "variable_target_unknown"),
        ("Scores['missing']", "1", "variable_target_unknown"),
        ("Text[0]", "'z'", "variable_path_unsupported"),
    ] {
        let failed = send(
            &mut server,
            &mut id,
            "expression.set",
            json!({"frame_id":current,"target":target,"expression":expression}),
        );
        assert_eq!(failed[0]["error"]["code"], code, "{target}: {failed:?}");
        let preserved = send(
            &mut server,
            &mut id,
            "evaluate",
            json!({"frame_id":current,"expression":"Counter"}),
        );
        assert_eq!(preserved[0]["body"]["result"], "41", "{preserved:?}");
    }

    for (target, code) in [
        ("State.Items[", "expression_target_parse"),
        ("Build()[0]", "expression_target_unsupported"),
    ] {
        let failed = send(
            &mut server,
            &mut id,
            "expression.set",
            json!({"frame_id":current,"target":target,"expression":"1"}),
        );
        assert_eq!(failed[0]["error"]["code"], code, "{failed:?}");
        assert!(
            failed[0]["error"]["help"]
                .as_str()
                .is_some_and(|help| !help.is_empty())
        );
        assert!(failed[0]["error"]["offset"].is_u64());
        assert!(failed[0]["error"]["length"].is_u64());
    }

    let local_without_frame = send(
        &mut server,
        &mut id,
        "expression.set",
        json!({"target":"Counter","expression":"1"}),
    );
    assert_eq!(
        local_without_frame[0]["error"]["code"],
        "variable_target_unknown"
    );

    let global = send(
        &mut server,
        &mut id,
        "expression.set",
        json!({"target":"GlobalValue","expression":"99"}),
    );
    assert_eq!(global[0]["body"]["result"], "99", "{global:?}");

    let _ = send(&mut server, &mut id, "continue", json!({}));
    let terminated = server.wait();
    let output = terminated
        .iter()
        .filter(|record| record["event"] == "output")
        .filter_map(|record| record["body"]["text"].as_str())
        .collect::<String>();
    assert_eq!(output, "99\n43\n42\n43\nabc\n");
}

#[test]
fn jsonl_expression_set_validates_arguments_and_lifecycle() {
    let mut created = new_server();
    let invalid = created.handle_line(&request(
        1,
        "expression.set",
        json!({"target":"GlobalValue","expression":"1"}),
    ));
    assert_eq!(invalid[0]["error"]["code"], "invalid_state", "{invalid:?}");

    let mut server = new_server();
    let mut id = 0;
    let _ = send(&mut server, &mut id, "initialize", json!({}));
    let initialized = send(
        &mut server,
        &mut id,
        "expression.set",
        json!({"target":"GlobalValue","expression":"1"}),
    );
    assert_eq!(initialized[0]["error"]["code"], "invalid_state");

    let mut running = new_server();
    let _ = running.handle_line(&request(1, "initialize", json!({})));
    let _ = running.handle_line(&request(2, "launch", json!({"stop_on_entry":false})));
    let invalid = running.handle_line(&request(
        3,
        "expression.set",
        json!({"target":"GlobalValue","expression":"1"}),
    ));
    assert_eq!(invalid[0]["error"]["code"], "invalid_state", "{invalid:?}");
    let _ = running.handle_line(&request(4, "disconnect", json!({})));

    let _ = send(
        &mut server,
        &mut id,
        "launch",
        json!({"stop_on_entry":true}),
    );

    for arguments in [
        json!({"expression":"1"}),
        json!({"target":"GlobalValue"}),
        json!({"target":1,"expression":"1"}),
        json!({"target":"GlobalValue","expression":1}),
        json!({"target":"GlobalValue","expression":"1","frame_id":"top"}),
        json!({"target":"GlobalValue","expression":"1","frame_id":-1}),
    ] {
        let failed = send(&mut server, &mut id, "expression.set", arguments);
        assert_eq!(failed[0]["error"]["code"], "invalid_request", "{failed:?}");
        assert!(
            failed[0]["error"]["help"]
                .as_str()
                .is_some_and(|help| !help.is_empty())
        );
    }

    let _ = send(&mut server, &mut id, "disconnect", json!({}));
    let terminated = send(
        &mut server,
        &mut id,
        "expression.set",
        json!({"target":"GlobalValue","expression":"1"}),
    );
    assert_eq!(
        terminated[0]["body"]["code"], "invalid_state",
        "{terminated:?}"
    );
}
