//! Actual rich-query routes, shared identity, UTF-16 conversion and dirty dependencies.

#![allow(
    clippy::expect_used,
    reason = "protocol fixtures assert concrete replies"
)]

use crate::support::{self, TempDirectory, TranscriptStep};
use fpas_language_service::{CancellationToken, SourceQueryOperation, SourceSession};
use serde_json::{Value, json};

fn open(uri: &str, text: &str) -> Value {
    json!({"jsonrpc":"2.0","method":"textDocument/didOpen","params":{"textDocument":{"uri":uri,"languageId":"fpas","version":1,"text":text}}})
}

fn query(id: i64, operation: &str, context: &str, uri: &str, source: &str, name: &str) -> Value {
    let offset = source.find(name).expect("query position");
    json!({"jsonrpc":"2.0","id":id,"method":"fpas/sourceQuery","params":{
        "version":1,"operation":operation,"context":context,
        "selector":{"kind":"position","uri":uri,"position":{"line":0,"character":source[..offset].encode_utf16().count()}}
    }})
}

#[test]
fn rich_find_and_selectors_preserve_ambiguity_and_editor_catalogue_policy() {
    let temp = TempDirectory::new("source-tools-find");
    temp.write(
        "app.fpasprj",
        "[project]\nname='app'\nkind='library'\n[sources]\ninclude=['units/*.fpas']\n",
    );
    temp.write(
        "units/first.fpas",
        "unit First; public function Answer(): integer; begin return 1; end function; end unit;",
    );
    temp.write(
        "units/second.fpas",
        "unit Second; public function Answer(): integer; begin return 2; end function; end unit;",
    );
    temp.write("loose.fpas", "program Loose; begin end.");
    let context = temp.uri("app.fpasprj");
    let request = |id, operation, selector, limit| {
        json!({"jsonrpc":"2.0","id":id,"method":"fpas/sourceQuery","params":{
            "version":1,"operation":operation,"context":context,"selector":selector,"limit":limit
        }})
    };
    let mut catalogue = request(7, "find", json!({"kind":"search","text":""}), 100);
    catalogue["params"]["policy"] = json!("editor_catalog");
    let transcript = support::run_script(&[
        TranscriptStep::Message(support::initialize_with_root(1, Some(&temp.uri("")))),
        TranscriptStep::Message(open(
            &temp.uri("loose.fpas"),
            "program Loose; const DirtyOnly: integer := 1; begin end.",
        )),
        TranscriptStep::Message(request(
            2,
            "find",
            json!({"kind":"search","text":"Answer"}),
            1,
        )),
        TranscriptStep::Message(request(
            3,
            "definition",
            json!({"kind":"symbol","name":"Answer"}),
            1,
        )),
        TranscriptStep::Message(request(
            4,
            "type",
            json!({"kind":"symbol","name":"First.Answer"}),
            100,
        )),
        TranscriptStep::MessageFrom(Box::new(|messages| {
            json!({"jsonrpc":"2.0","id":5,"method":"fpas/sourceQuery","params":{
                "version":1,"operation":"definition","selector":{"kind":"reference","target_ref":support::response(messages,2)["result"]["result"]["symbols"][0]["target_ref"]}
            }})
        })),
        TranscriptStep::Message(
            json!({"jsonrpc":"2.0","id":6,"method":"workspace/symbol","params":{"query":""}}),
        ),
        TranscriptStep::Message(catalogue),
        TranscriptStep::Message(request(
            8,
            "find",
            json!({"kind":"search","text":"DirtyOnly"}),
            100,
        )),
        TranscriptStep::Message(request(
            10,
            "find",
            json!({"kind":"search","text":"integer"}),
            10,
        )),
        TranscriptStep::Message(support::shutdown(9)),
        TranscriptStep::Message(support::exit()),
    ]);
    let found = &support::response(&transcript.messages, 2)["result"];
    assert_eq!(found["outcome"], "incomplete", "{found}");
    assert_eq!(found["coverage"]["analysis_complete"], true);
    assert_eq!(
        found["coverage"]["collections"]["symbols"],
        json!({"returned":1,"known_total":2,"complete":false})
    );
    let ambiguous = &support::response(&transcript.messages, 3)["result"];
    assert_eq!(ambiguous["outcome"], "error", "{ambiguous}");
    assert_eq!(ambiguous["diagnostics"][0]["code"], "ambiguous_target");
    assert!(ambiguous["target"].is_null());
    assert_eq!(
        support::response(&transcript.messages, 4)["result"]["result"]["type"]["kind"],
        "callable"
    );
    assert_eq!(
        support::response(&transcript.messages, 5)["result"]["target"]["target_ref"],
        found["result"]["symbols"][0]["target_ref"]
    );
    let catalogue = &support::response(&transcript.messages, 7)["result"];
    assert_eq!(catalogue["context"]["policy"], "editor_catalog");
    let symbols = catalogue["result"]["symbols"]
        .as_array()
        .expect("catalogue symbols");
    assert!(
        symbols
            .iter()
            .all(|s| s["target_ref"].is_null() && s["checked"] == false)
    );
    assert!(symbols.iter().any(|s| s["name"] == "DirtyOnly"));
    let standard = support::response(&transcript.messages, 6)["result"]
        .as_array()
        .expect("standard symbols");
    assert_eq!(
        symbols.iter().map(|s| &s["name"]).collect::<Vec<_>>(),
        standard.iter().map(|s| &s["name"]).collect::<Vec<_>>()
    );
    assert_eq!(
        support::response(&transcript.messages, 8)["result"]["result"]["symbols"],
        json!([])
    );
    let builtin = &support::response(&transcript.messages, 10)["result"]["result"]["symbols"][0];
    assert_eq!(builtin["name"], "integer");
    assert_eq!(builtin["origin"], "builtin");
    assert!(builtin["declaration"].is_null() && builtin["target_ref"].is_null());
    assert_eq!(builtin["reference_unavailable"], "no_source");
}

#[test]
fn rich_query_matches_core_identity_and_rejects_a_reference_after_an_unedited_input_changes() {
    let temp = TempDirectory::new("source-tools-reference");
    let manifest = temp.write(
        "app.fpasprj",
        "[project]\nname='app'\nkind='program'\nmain='main.fpas'\n[sources]\ninclude=['*.fpas']\n",
    );
    let source = "program Main; uses Demo.Core as C; begin discard '😀'; discard C.Answer(); end.";
    let main = temp.write("main.fpas", source);
    let unit = temp.write("core.fpas", "unit Demo.Core; public function Answer(): integer; begin return 42; end function; end unit;");
    let cancel = CancellationToken::new();
    let mut core = SourceSession::load(&manifest, None, &[], &cancel).expect("core scope");
    let expected = core
        .query_position(
            &main,
            source.find("Answer").expect("call"),
            SourceQueryOperation::Definition,
            1000,
            &cancel,
        )
        .expect("core query")
        .target
        .expect("core target")
        .target_ref
        .expect("portable reference");
    drop(core);
    let context = temp.uri("app.fpasprj");
    let uri = temp.uri("main.fpas");
    let mut bad = query(5, "definition", &context, &uri, source, "Answer");
    bad["params"]["version"] = json!(9);
    let transcript = support::run_script(&[
        TranscriptStep::Message(support::initialize_with_root(1, Some(&temp.uri("")))),
        TranscriptStep::Message(open(&uri, source)),
        TranscriptStep::Message(query(2, "definition", &context, &uri, source, "Answer")),
        TranscriptStep::MessageFrom(Box::new(|messages| {
            json!({"jsonrpc":"2.0","id":3,"method":"fpas/sourceQuery","params":{
                "version":1,"operation":"refs","selector":{"kind":"reference","target_ref":support::response(messages,2)["result"]["target"]["target_ref"]}
            }})
        })),
        TranscriptStep::Action(Box::new(move || {
            std::fs::write(&unit, "unit Demo.Core; public function Answer(): integer; begin return 43; end function; end unit;").expect("changed dependency")
        })),
        TranscriptStep::MessageFrom(Box::new(|messages| {
            json!({"jsonrpc":"2.0","id":4,"method":"fpas/sourceQuery","params":{
                "version":1,"operation":"refs","selector":{"kind":"reference","target_ref":support::response(messages,2)["result"]["target"]["target_ref"]}
            }})
        })),
        TranscriptStep::Message(bad),
        TranscriptStep::Message(support::shutdown(6)),
        TranscriptStep::Message(support::exit()),
    ]);
    assert!(transcript.output.status.success());
    let result = &support::response(&transcript.messages, 2)["result"];
    assert_eq!(result["outcome"], "ok", "{result}");
    assert_eq!(
        result["target"]["target_ref"],
        serde_json::to_value(expected).expect("reference JSON")
    );
    let refs = &support::response(&transcript.messages, 3)["result"];
    assert_eq!(
        refs["result"]["locations"][0]["range"]["start"]["character"],
        source[..source.find("Answer").expect("call")]
            .encode_utf16()
            .count()
    );
    assert_eq!(
        support::response(&transcript.messages, 4)["error"]["data"]["code"],
        "stale_target"
    );
    assert_eq!(
        support::response(&transcript.messages, 5)["error"]["code"],
        -32602
    );
    assert_eq!(
        support::response(&transcript.messages, 1)["result"]["capabilities"]["experimental"]["fpasSourceQuery"]
            ["version"],
        1
    );
}

#[test]
fn rich_type_uses_dirty_callers_and_dependencies_without_saving() {
    let temp = TempDirectory::new("source-tools-overlay");
    temp.write(
        "app.fpasprj",
        "[project]\nname='app'\nkind='program'\nmain='main.fpas'\n[sources]\ninclude=['*.fpas']\n",
    );
    let saved_main =
        "program Main; uses Demo.Core as C; begin const Value: integer := C.Answer(); end.";
    let saved_unit = "unit Demo.Core; public function Answer(): integer; begin return 42; end function; end unit;";
    let main = temp.write("main.fpas", saved_main);
    let unit = temp.write("core.fpas", saved_unit);
    let dirty_main = saved_main.replace("integer", "string");
    let dirty_unit = saved_unit
        .replace("integer", "string")
        .replace("42", "'dirty'");
    let uri = temp.uri("main.fpas");
    let transcript = support::run(&[
        support::initialize_with_root(1, Some(&temp.uri(""))),
        open(&uri, &dirty_main),
        open(&temp.uri("core.fpas"), &dirty_unit),
        query(
            2,
            "type",
            &temp.uri("app.fpasprj"),
            &uri,
            &dirty_main,
            "Value",
        ),
        support::shutdown(3),
        support::exit(),
    ]);
    let result = &support::response(&transcript.messages, 2)["result"];
    assert_eq!(result["outcome"], "ok", "{result}");
    assert_eq!(result["result"]["type"]["display"], "string");
    assert_eq!(
        std::fs::read_to_string(main).expect("saved main"),
        saved_main
    );
    assert_eq!(
        std::fs::read_to_string(unit).expect("saved unit"),
        saved_unit
    );
}

#[test]
fn rich_inspect_uses_dirty_dependency_sections_with_exact_utf16_locations() {
    let temp = TempDirectory::new("source-tools-inspect");
    temp.write(
        "app.fpasprj",
        "[project]\nname='app'\nkind='program'\nmain='main.fpas'\n[sources]\ninclude=['*.fpas']\n",
    );
    let main = "program Main; uses Demo.Core as C; begin discard C.Answer(); end.";
    temp.write("main.fpas", main);
    let saved = "unit Demo.Core; public function Answer(): integer; begin return 1; end function; end unit;";
    let path = temp.write("core.fpas", saved);
    let dirty = "unit Demo.Core; // 😀\r\n// Unsaved documentation.\r\npublic function Answer(): integer; begin return 73; end function; end unit;";
    let transcript = support::run(&[
        support::initialize_with_root(1, Some(&temp.uri(""))),
        open(&temp.uri("main.fpas"), main),
        open(&temp.uri("core.fpas"), dirty),
        query(
            2,
            "inspect",
            &temp.uri("app.fpasprj"),
            &temp.uri("main.fpas"),
            main,
            "Answer",
        ),
        support::shutdown(3),
        support::exit(),
    ]);
    let response = &support::response(&transcript.messages, 2)["result"];
    assert_eq!(response["outcome"], "ok", "{response}");
    let sections = response["result"]["sections"]
        .as_array()
        .expect("inspection sections");
    assert_eq!(sections[0]["documentation"], "Unsaved documentation.");
    assert!(
        sections[1]["excerpts"][0]["text"]
            .as_str()
            .expect("body")
            .contains("return 73")
    );
    for section in sections {
        for excerpt in section["excerpts"].as_array().expect("excerpts") {
            let start = excerpt["location"]["byte_start"].as_u64().expect("start") as usize;
            let length = excerpt["location"]["byte_length"].as_u64().expect("length") as usize;
            assert_eq!(excerpt["text"], &dirty[start..start + length]);
            let line_start = dirty[..start].rfind('\n').map_or(0, |i| i + 1);
            assert_eq!(
                excerpt["location"]["range"]["start"]["character"],
                dirty[line_start..start].encode_utf16().count()
            );
        }
    }
    assert_eq!(
        std::fs::read_to_string(path).expect("saved dependency"),
        saved
    );
}

#[test]
fn deleted_open_buffers_outside_the_requested_project_do_not_break_inspection() {
    let temp = TempDirectory::new("source-tools-unrelated-deletion");
    temp.write(
        "app.fpasprj",
        "[project]\nname='app'\nkind='program'\nmain='main.fpas'\n[sources]\ninclude=['*.fpas']\n",
    );
    let main =
        "program Main; function Answer(): integer; begin return 42; end function; begin end.";
    temp.write("main.fpas", main);
    let old = temp.write("old/gone.fpas", "program Old; begin end.");
    let transcript = support::run_script(&[
        TranscriptStep::Message(support::initialize_with_root(1, Some(&temp.uri("")))),
        TranscriptStep::Message(open(&temp.uri("main.fpas"), main)),
        TranscriptStep::Message(open(&temp.uri("old/gone.fpas"), "program Old; begin end.")),
        TranscriptStep::Message(query(
            2,
            "definition",
            &temp.uri("app.fpasprj"),
            &temp.uri("main.fpas"),
            main,
            "Answer",
        )),
        TranscriptStep::Action(Box::new(move || {
            std::fs::remove_file(&old).expect("external deletion")
        })),
        TranscriptStep::Message(query(
            3,
            "inspect",
            &temp.uri("app.fpasprj"),
            &temp.uri("main.fpas"),
            main,
            "Answer",
        )),
        TranscriptStep::Message(support::shutdown(4)),
        TranscriptStep::Message(support::exit()),
    ]);
    let response = &support::response(&transcript.messages, 3)["result"];
    assert_eq!(response["outcome"], "ok", "{response}");
    assert!(
        response["result"]["sections"][1]["excerpts"][0]["text"]
            .as_str()
            .expect("body")
            .contains("return 42")
    );
}
