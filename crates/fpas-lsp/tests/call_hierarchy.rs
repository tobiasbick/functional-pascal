//! Actual hierarchy routes share source-query edges and reject stale editor identities.
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
fn prepare(id: i64, uri: &str, text: &str, needle: &str) -> Value {
    let start = text.find(needle).expect("source position");
    json!({"jsonrpc":"2.0","id":id,"method":"textDocument/prepareCallHierarchy","params":{"textDocument":{"uri":uri},"position":{"line":0,"character":text[..start].encode_utf16().count()}}})
}
fn calls(id: i64, method: &str, item: &Value) -> Value {
    json!({"jsonrpc":"2.0","id":id,"method":method,"params":{"item":item}})
}

#[test]
fn hierarchy_and_rich_calls_match_core_sites_and_reject_dirty_stale_items() {
    let temp = TempDirectory::new("call-hierarchy");
    let manifest = temp.write(
        "app.fpasprj",
        "[project]\nname='app'\nkind='program'\nmain='main.fpas'\n[sources]\ninclude=['*.fpas']\n",
    );
    let source = "program Main; function Leaf(Value: integer): integer; begin return Value; end function; function Parent(): integer; begin const F := Leaf; discard '😀'; discard Leaf(1); discard Leaf(Value := 2); return F(3); end function; begin discard Parent(); end.";
    let path = temp.write("main.fpas", source);
    let cancel = CancellationToken::new();
    let mut core = SourceSession::load(&manifest, None, &[], &cancel).expect("core scope");
    let expected = core
        .query_position(
            &path,
            source.find("Parent()").expect("parent"),
            SourceQueryOperation::Callees,
            1000,
            &cancel,
        )
        .expect("core callees")
        .calls
        .expect("relations");
    let expected_ranges = expected
        .direct_calls
        .items
        .iter()
        .map(|call| {
            json!({
                "start":{"line":0,"character":source[..call.span.offset()].encode_utf16().count()},
                "end":{"line":0,"character":source[..call.span.end()].encode_utf16().count()}
            })
        })
        .collect::<Vec<_>>();
    drop(core);
    let dirty = source.replace("return F(3)", "discard Leaf(9); return F(3)");
    let uri = temp.uri("main.fpas");
    let rich = |id| {
        json!({"jsonrpc":"2.0","id":id,"method":"fpas/sourceQuery","params":{
        "version":1,"operation":"callees","context":temp.uri("app.fpasprj"),"selector":{"kind":"symbol","name":"Parent"}}})
    };
    let transcript = support::run_script(&[
        TranscriptStep::Message(support::initialize_with_root(1, Some(&temp.uri("")))),
        TranscriptStep::Message(open(&uri, source)),
        TranscriptStep::Message(prepare(2, &uri, source, "Leaf(Value")),
        TranscriptStep::MessageFrom(Box::new(|messages| {
            calls(
                3,
                "callHierarchy/incomingCalls",
                &support::response(messages, 2)["result"][0],
            )
        })),
        TranscriptStep::Message(prepare(4, &uri, source, "Parent()")),
        TranscriptStep::MessageFrom(Box::new(|messages| {
            calls(
                5,
                "callHierarchy/outgoingCalls",
                &support::response(messages, 4)["result"][0],
            )
        })),
        TranscriptStep::Message(rich(6)),
        TranscriptStep::Message(
            json!({"jsonrpc":"2.0","method":"textDocument/didChange","params":{"textDocument":{"uri":uri,"version":2},"contentChanges":[{"text":dirty}]}}),
        ),
        TranscriptStep::MessageFrom(Box::new(|messages| {
            calls(
                7,
                "callHierarchy/outgoingCalls",
                &support::response(messages, 4)["result"][0],
            )
        })),
        TranscriptStep::Message(prepare(8, &uri, &dirty, "Parent()")),
        TranscriptStep::MessageFrom(Box::new(|messages| {
            calls(
                9,
                "callHierarchy/outgoingCalls",
                &support::response(messages, 8)["result"][0],
            )
        })),
        TranscriptStep::Message(rich(10)),
        TranscriptStep::Message(support::shutdown(11)),
        TranscriptStep::Message(support::exit()),
    ]);
    assert_eq!(
        support::response(&transcript.messages, 1)["result"]["capabilities"]["callHierarchyProvider"],
        true
    );
    assert!(
        support::response(&transcript.messages, 2)["result"].is_array(),
        "prepare: {}",
        support::response(&transcript.messages, 2)
    );
    assert!(
        support::response(&transcript.messages, 3)["result"].is_array(),
        "incoming: {}",
        support::response(&transcript.messages, 3)
    );
    let incoming = &support::response(&transcript.messages, 3)["result"];
    assert_eq!(
        incoming.as_array().expect("incoming").len(),
        1,
        "{incoming}"
    );
    assert_eq!(incoming[0]["from"]["name"], "Main.Parent");
    assert_eq!(incoming[0]["fromRanges"], json!(expected_ranges));
    let outgoing = &support::response(&transcript.messages, 5)["result"];
    assert_eq!(outgoing[0]["to"]["name"], "Main.Leaf", "{outgoing}");
    assert_eq!(outgoing[0]["fromRanges"], json!(expected_ranges));
    let saved = &support::response(&transcript.messages, 6)["result"];
    assert_eq!(saved["result"]["direct_edges_complete"], true, "{saved}");
    assert_eq!(saved["result"]["runtime_targets_complete"], false);
    assert_eq!(
        saved["result"]["dynamic_calls"]
            .as_array()
            .expect("dynamic")
            .len(),
        1
    );
    assert_eq!(
        saved["result"]["direct_calls"]
            .as_array()
            .expect("direct")
            .iter()
            .map(|c| c["location"]["range"].clone())
            .collect::<Vec<_>>(),
        expected_ranges
    );
    assert_eq!(
        support::response(&transcript.messages, 7)["error"]["data"]["code"],
        "stale_target"
    );
    assert_eq!(
        support::response(&transcript.messages, 9)["result"][0]["fromRanges"]
            .as_array()
            .expect("dirty ranges")
            .len(),
        3
    );
    let changed = &support::response(&transcript.messages, 10)["result"];
    assert_eq!(
        changed["result"]["direct_calls"]
            .as_array()
            .expect("dirty direct")
            .len(),
        3
    );
    assert_eq!(std::fs::read_to_string(path).expect("saved source"), source);
}

#[test]
fn hierarchy_items_for_retained_defaults_and_closures_can_be_followed() {
    let temp = TempDirectory::new("call-hierarchy-owners");
    temp.write(
        "app.fpasprj",
        "[project]\nname='app'\nkind='program'\nmain='main.fpas'\n[sources]\ninclude=['*.fpas']\n",
    );
    let source = "program Main; function Seed(): integer; begin return 7; end function; type Box = record Value: integer := Seed(); end record; function Outer(): integer; begin const F := function(): integer begin return Seed(); end function; return F(); end function; begin const A := Box(); const B := Box(); discard Outer(); end.";
    temp.write("main.fpas", source);
    let uri = temp.uri("main.fpas");
    let transcript = support::run_script(&[
        TranscriptStep::Message(support::initialize_with_root(1, Some(&temp.uri("")))),
        TranscriptStep::Message(open(&uri, source)),
        TranscriptStep::Message(prepare(2, &uri, source, "Seed()")),
        TranscriptStep::MessageFrom(Box::new(|messages| {
            calls(
                3,
                "callHierarchy/incomingCalls",
                &support::response(messages, 2)["result"][0],
            )
        })),
        TranscriptStep::MessageFrom(Box::new(|messages| {
            calls(
                4,
                "callHierarchy/outgoingCalls",
                &support::response(messages, 3)["result"][0]["from"],
            )
        })),
        TranscriptStep::MessageFrom(Box::new(|messages| {
            calls(
                5,
                "callHierarchy/outgoingCalls",
                &support::response(messages, 3)["result"][1]["from"],
            )
        })),
        TranscriptStep::Message(support::shutdown(6)),
        TranscriptStep::Message(support::exit()),
    ]);
    let incoming_response = support::response(&transcript.messages, 3);
    assert!(
        incoming_response["result"].is_array(),
        "prepare: {}; incoming: {incoming_response}",
        support::response(&transcript.messages, 2)
    );
    let incoming = &incoming_response["result"];
    assert_eq!(
        incoming.as_array().expect("incoming").len(),
        2,
        "{incoming}"
    );
    assert_eq!(
        incoming[0]["from"]["data"]["callable"]["kind"],
        "record_default"
    );
    assert_eq!(incoming[1]["from"]["data"]["callable"]["kind"], "closure");
    for id in [4, 5] {
        assert_eq!(
            support::response(&transcript.messages, id)["result"][0]["to"]["name"],
            "Main.Seed"
        );
    }
}
