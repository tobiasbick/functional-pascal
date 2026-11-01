//! Shared recipes over the actual LSP process, including stale read sets and client refusal.

#![allow(
    clippy::expect_used,
    reason = "protocol fixtures assert concrete replies"
)]

use crate::support::{self, TempDirectory, TranscriptStep as Step};
use fpas_language_service::{CancellationToken, RefactorRecipe, SourceSelector, SourceSession};
use serde_json::{Value, json};

fn open(uri: &str, source: &str, version: i32) -> Value {
    json!({"jsonrpc":"2.0","method":"textDocument/didOpen","params":{"textDocument":{"uri":uri,"languageId":"fpas","version":version,"text":source}}})
}
fn change(uri: &str, source: &str, version: i32) -> Value {
    json!({"jsonrpc":"2.0","method":"textDocument/didChange","params":{"textDocument":{"uri":uri,"version":version},"contentChanges":[{"text":source}]}})
}
fn prepare(id: i64, context: &str, recipe: Value) -> Value {
    json!({"jsonrpc":"2.0","id":id,"method":"fpas/refactor","params":{"version":1,"phase":"prepare","context":context,"recipe":recipe}})
}
fn reuse(id: i64, original: i64, command: bool) -> Step {
    Step::MessageFrom(Box::new(move |messages| {
        let handle = &support::response(messages, original)["result"]["handle"];
        if command {
            json!({"jsonrpc":"2.0","id":id,"method":"workspace/executeCommand","params":{"command":"fpas.applyRefactor","arguments":[handle]}})
        } else {
            json!({"jsonrpc":"2.0","id":id,"method":"fpas/refactor","params":{"version":1,"phase":"revalidate","handle":handle}})
        }
    }))
}
fn fixture(label: &str) -> TempDirectory {
    let temp = TempDirectory::new(label);
    temp.write(
        "app.fpasprj",
        "[project]\nname='app'\nkind='program'\nmain='main.fpas'\n[sources]\ninclude=['*.fpas']\n",
    );
    temp.write(
        "value.fpas",
        "unit Demo.Value; public function Seed(): integer; begin return 1; end function; end unit;",
    );
    temp
}

#[test]
fn parameter_recipe_shares_core_edits_and_applies_dirty_unicode_callers_with_versions() {
    use fpas_language_service::ParameterMode;
    let temp = fixture("refactor-parameter");
    let saved =
        "program Demo; uses Demo.Value as Value; begin discard '😀'; discard Value.Seed(); end.";
    let dirty = saved.replace(
        "discard Value.Seed();",
        "discard Value.Seed(); discard Value.Seed();",
    );
    temp.write("main.fpas", saved);
    let context = temp.uri("app.fpasprj");
    let uri = temp.uri("main.fpas");
    let unit_uri = temp.uri("value.fpas");
    let request = json!({"action":"add-parameter","selector":{"kind":"symbol","name":"Demo.Value.Seed"},"name":"Extra","type":"integer","mode":"value","argument":"9"});
    let mut cursor_request = request.clone();
    cursor_request["selector"] = json!({"kind":"position","uri":uri,"position":{"line":0,"character":dirty[..dirty.find("Seed()").expect("call")].encode_utf16().count()}});
    let transcript = support::run_script(&[
        Step::Message(support::initialize_with_root(1, Some(&context))),
        Step::Message(prepare(2, &context, request)),
        Step::Message(open(&uri, &dirty, 7)),
        Step::Message(open(
            &unit_uri,
            "unit Demo.Value; public function Seed(): integer; begin return 2; end function; end unit;",
            5,
        )),
        Step::Message(prepare(3, &context, cursor_request)),
        reuse(4, 3, false),
        reuse(5, 3, true),
        Step::Message(change(&uri, &dirty, 8)),
        reuse(6, 3, true),
        Step::Message(support::shutdown(7)),
        Step::Message(support::exit()),
    ]);
    let result = &support::response(&transcript.messages, 2)["result"];
    assert_eq!(result["status"], "prepared", "{result}");
    let cancel = CancellationToken::new();
    let mut session =
        SourceSession::load(&temp.path().join("app.fpasprj"), None, &[], &cancel).expect("core");
    let core = session
        .prepare_refactor(
            &RefactorRecipe::AddParameter {
                selector: SourceSelector::Symbol {
                    name: "Demo.Value.Seed".into(),
                },
                name: "Extra".into(),
                r#type: "integer".into(),
                mode: ParameterMode::Value,
                argument: "9".into(),
            },
            &cancel,
        )
        .expect("core prepare");
    let editor_texts = result["edit"]["documentChanges"]
        .as_array()
        .expect("changes")
        .iter()
        .flat_map(|c| {
            c["edits"]
                .as_array()
                .expect("edits")
                .iter()
                .map(|e| e["newText"].as_str().expect("text").to_owned())
        })
        .collect::<Vec<_>>();
    assert_eq!(
        editor_texts,
        core.changes
            .expect("core changes")
            .edits
            .into_iter()
            .map(|e| e.new_text)
            .collect::<Vec<_>>()
    );
    let dirty_result = &support::response(&transcript.messages, 3)["result"];
    assert_eq!(dirty_result["status"], "prepared", "{dirty_result}");
    let changes = dirty_result["edit"]["documentChanges"]
        .as_array()
        .expect("dirty changes");
    assert!(changes.iter().any(|c| c["textDocument"]["uri"] == uri
        && c["textDocument"]["version"] == 7
        && c["edits"].as_array().expect("calls").len() == 2));
    assert!(
        changes
            .iter()
            .any(|c| c["textDocument"]["uri"] == unit_uri && c["textDocument"]["version"] == 5)
    );
    assert_eq!(
        dirty_result["edit"],
        support::response(&transcript.messages, 4)["result"]["edit"]
    );
    assert_eq!(
        support::response(&transcript.messages, 5)["result"]["status"],
        "applied"
    );
    assert_eq!(
        support::response(&transcript.messages, 6)["error"]["data"]["code"],
        "stale_plan"
    );
    assert_eq!(
        transcript
            .messages
            .iter()
            .filter(|m| m["method"] == "workspace/applyEdit")
            .count(),
        1
    );
    assert_eq!(
        std::fs::read_to_string(temp.path().join("main.fpas")).expect("unsaved"),
        saved
    );
}

#[test]
fn saved_rename_matches_core_and_standard_route_while_dirty_versions_stay_guarded() {
    let temp = fixture("refactor-parity");
    let source = "program Demo; uses Demo.Value as Value; begin const Total := Value.Seed(); end.";
    temp.write("main.fpas", source);
    let context = temp.uri("app.fpasprj");
    let uri = temp.uri("main.fpas");
    let selector = json!({"kind":"position","uri":uri,"position":{"line":0,"character":source.find("Total").expect("name")}});
    let recipe = json!({"action":"rename","selector":selector,"name":"Sum"});
    let transcript = support::run_script(&[
        Step::Message(support::initialize_with_root(1, Some(&context))),
        Step::Message(open(&uri, source, 4)),
        Step::Message(prepare(2, &context, recipe.clone())),
        Step::Message(
            json!({"jsonrpc":"2.0","id":3,"method":"textDocument/rename","params":{"textDocument":{"uri":uri},"position":selector["position"],"newName":"Sum"}}),
        ),
        reuse(4, 2, false),
        Step::Message(change(&uri, source, 5)),
        reuse(5, 2, false),
        Step::Message(prepare(
            6,
            &context,
            json!({"action":"rename","selector":selector,"name":"begin"}),
        )),
        Step::Message(support::shutdown(7)),
        Step::Message(support::exit()),
    ]);
    let result = &support::response(&transcript.messages, 2)["result"];
    assert_eq!(result["status"], "prepared", "{result}");
    assert_eq!(
        result["edit"],
        support::response(&transcript.messages, 3)["result"]
    );
    assert_eq!(
        result["edit"],
        support::response(&transcript.messages, 4)["result"]["edit"]
    );
    assert_eq!(
        result["edit"]["documentChanges"][0]["textDocument"]["version"],
        4
    );
    assert_eq!(
        support::response(&transcript.messages, 5)["error"]["data"]["code"],
        "stale_plan"
    );
    let unsupported = &support::response(&transcript.messages, 6)["result"];
    assert_eq!(unsupported["status"], "unsupported");
    assert!(unsupported["edit"].is_null() && unsupported["handle"].is_null());
    let token = CancellationToken::new();
    let mut session =
        SourceSession::load(&temp.path().join("app.fpasprj"), None, &[], &token).expect("core");
    let prepared = session
        .prepare_refactor(
            &RefactorRecipe::Rename {
                selector: SourceSelector::Symbol {
                    name: "Total".into(),
                },
                name: "Sum".into(),
            },
            &token,
        )
        .expect("prepare");
    assert_eq!(result["snapshot"]["id"], session.snapshot().id);
    let changes = prepared.changes.expect("checked core edits");
    assert_eq!(changes.edits.len(), 1);
    assert_eq!(
        result["edit"]["documentChanges"][0]["edits"][0]["newText"],
        changes.edits[0].new_text
    );
}

#[test]
fn cached_actions_reject_edited_or_closed_unedited_dependencies() {
    for kind in ["extract-variable", "organize-imports"] {
        for dirty in [false, true] {
            let temp = fixture("refactor-stale-dependency");
            let source = "program Demo;\nuses Demo.Value as Value;\nbegin\n  const Output := (1 + 2) * 3;\nend.";
            temp.write("main.fpas", source);
            let context = temp.uri("app.fpasprj");
            let uri = temp.uri("main.fpas");
            let offset = source.find("(1 + 2)").expect("expression");
            let column = offset - source[..offset].rfind('\n').expect("statement line") - 1;
            let action_kind = if kind == "extract-variable" {
                "refactor.extract"
            } else {
                "source.organizeImports"
            };
            let mut steps = vec![
                Step::Message(support::initialize_with_root(1, Some(&context))),
                Step::Message(open(&uri, source, 1)),
            ];
            if dirty {
                steps.push(Step::Message(open(
                    &temp.uri("value.fpas"),
                    &std::fs::read_to_string(temp.path().join("value.fpas")).expect("unit"),
                    1,
                )));
            }
            steps.push(Step::Message(json!({"jsonrpc":"2.0","id":2,"method":"textDocument/codeAction","params":{"textDocument":{"uri":uri},"range":{"start":{"line":3,"character":column},"end":{"line":3,"character":column+7}},"context":{"diagnostics":[],"only":[action_kind]}}})));
            let changed = "unit Demo.Value; public function Seed(): integer; begin return 2; end function; end unit;";
            if dirty {
                steps.push(Step::Message(change(&temp.uri("value.fpas"), changed, 2)));
            } else {
                let file = temp.path().join("value.fpas");
                steps.push(Step::Action(Box::new(move || {
                    std::fs::write(&file, changed).expect("change dependency")
                })));
            }
            steps.push(Step::MessageFrom(Box::new(|messages| {
                let actions = &support::response(messages, 2)["result"];
                assert_eq!(actions.as_array().expect("actions").len(), 1, "{actions}");
                assert!(actions[0].get("edit").is_none());
                json!({"jsonrpc":"2.0","id":3,"method":"workspace/executeCommand","params":{"command":"fpas.applyRefactor","arguments":actions[0]["command"]["arguments"]}})
            })));
            steps.extend([
                Step::Message(support::shutdown(4)),
                Step::Message(support::exit()),
            ]);
            let transcript = support::run_script(&steps);
            assert_eq!(
                support::response(&transcript.messages, 3)["error"]["data"]["code"],
                "stale_plan",
                "{}",
                support::response(&transcript.messages, 3)
            );
            assert!(
                !transcript
                    .messages
                    .iter()
                    .any(|m| m["method"] == "workspace/applyEdit")
            );
            assert_eq!(
                std::fs::read_to_string(temp.path().join("main.fpas")).expect("source"),
                source
            );
        }
    }
}

#[test]
fn client_rejection_is_reported_and_missing_apply_capability_omits_actions() {
    let temp = fixture("refactor-client-rejection");
    let source = "program Demo; uses Demo.Value as Value; begin end.";
    temp.write("main.fpas", source);
    let context = temp.uri("app.fpasprj");
    let uri = temp.uri("main.fpas");
    let transcript = support::run_script(&[
        Step::Message(support::initialize_with_root(1, Some(&context))),
        Step::Message(open(&uri, source, 1)),
        Step::Message(prepare(
            2,
            &context,
            json!({"action":"organize-imports","uri":uri}),
        )),
        Step::RespondTo(
            "workspace/applyEdit",
            json!({"applied":false,"failureReason":"user kept the old text","failedChange":0}),
        ),
        reuse(3, 2, true),
        Step::Message(support::shutdown(4)),
        Step::Message(support::exit()),
    ]);
    assert_eq!(
        support::response(&transcript.messages, 3)["error"]["data"]["code"],
        "editor_rejected"
    );
    assert_eq!(
        std::fs::read_to_string(temp.path().join("main.fpas")).expect("source"),
        source
    );
    let mut initialize = support::initialize_with_root(1, Some(&context));
    initialize["params"]["capabilities"]["workspace"]["applyEdit"] = json!(false);
    let transcript = support::run(&[
        initialize,
        open(&uri, source, 1),
        json!({"jsonrpc":"2.0","id":2,"method":"textDocument/codeAction","params":{"textDocument":{"uri":uri},"range":{"start":{"line":0,"character":0},"end":{"line":0,"character":0}},"context":{"diagnostics":[],"only":["source.organizeImports"]}}}),
        support::shutdown(3),
        support::exit(),
    ]);
    assert_eq!(
        support::response(&transcript.messages, 2)["result"],
        json!([])
    );
}
