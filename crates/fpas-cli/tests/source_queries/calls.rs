//! Public call queries distinguish complete empty bodies, clipped direct edges and dynamic calls.
use super::*;

#[test]
fn callers_and_callees_preserve_exact_sites_independent_counts_and_portable_selection() {
    let source = "program Main; function Leaf(Value: integer): integer; begin return Value; end function; function Parent(): integer; begin const F := Leaf; discard '😀'; discard Leaf(1); discard Leaf(Value := 2); return F(3); end function; begin discard Parent(); end.";
    let f = Fixture::new(source);
    f.write(
        "app.fpasprj",
        "[project]\nname='app'\nkind='program'\nmain='main.fpas'\n[sources]\ninclude=['*.fpas']\n",
    );
    let report = f.run(
        &[
            "callers",
            "--symbol",
            "Leaf",
            "--context",
            "app.fpasprj",
            "--limit",
            "1",
        ],
        3,
    );
    assert_eq!(report["result"]["kind"], "callers");
    assert_eq!(report["result"]["direct_edges_complete"], true);
    assert_eq!(report["result"]["runtime_targets_complete"], false);
    assert_eq!(report["coverage"]["analysis_complete"], true);
    assert_eq!(report["limits"]["result_count"], 1);
    assert_eq!(
        report["coverage"]["collections"]["direct_calls"],
        json!({"returned":1,"known_total":2,"complete":false})
    );
    assert_eq!(
        report["coverage"]["collections"]["dynamic_calls"],
        json!({"returned":1,"known_total":1,"complete":true})
    );
    let call = &report["result"]["direct_calls"][0];
    let start = source.find("Leaf(1)").unwrap();
    assert_eq!(
        call["location"]["byte_span"],
        json!({"start":start,"length":7})
    );
    assert_eq!(
        call["location"]["range"]["start"]["column"],
        source[..start].chars().count() + 1
    );
    assert_eq!(
        call["containing"]["identity"]["qualified_name"],
        "Main.Parent"
    );
    assert!(report["target"]["target_ref"].is_object());
    f.write(
        "leaf.json",
        &serde_json::to_string(&report["target"]["target_ref"]).unwrap(),
    );
    let empty = f.run(&["callees", "--target-ref", "leaf.json"], 0);
    assert_eq!(empty["result"]["direct_calls"], json!([]));
    assert_eq!(empty["result"]["runtime_targets_complete"], true);
    assert_eq!(
        empty["coverage"]["collections"]["direct_calls"]["known_total"],
        0
    );
}

#[test]
fn native_targets_are_known_without_source_bodies_and_broken_sources_are_incomplete() {
    let f = Fixture::new(
        "program Main; uses Std.Io.Fs as Fs; begin discard Fs.ReadText('unused'); discard 'text'.Length(); end.",
    );
    f.write(
        "app.fpasprj",
        "[project]\nname='app'\nkind='program'\nmain='main.fpas'\n[sources]\ninclude=['*.fpas']\n",
    );
    let calls = f.run(
        &[
            "callers",
            "--symbol",
            "Std.Io.Fs.ReadText",
            "--context",
            "app.fpasprj",
        ],
        0,
    );
    assert_eq!(calls["result"]["direct_calls"].as_array().unwrap().len(), 1);
    assert_eq!(
        calls["result"]["direct_calls"][0]["target"]["identity"]["kind"],
        "builtin"
    );
    assert!(calls["result"]["direct_calls"][0]["target"]["location"].is_null());
    f.write(
        "native.json",
        &serde_json::to_string(&calls["target"]["target_ref"]).unwrap(),
    );
    let by_reference = f.run(&["callees", "--target-ref", "native.json"], 3);
    assert!(
        by_reference["coverage"]["reasons"]
            .as_array()
            .unwrap()
            .contains(&json!("body_unavailable")),
        "{by_reference}"
    );
    let no_body = f.run(
        &[
            "callees",
            "--symbol",
            "Std.Io.Fs.ReadText",
            "--context",
            "app.fpasprj",
        ],
        3,
    );
    assert!(
        no_body["coverage"]["reasons"]
            .as_array()
            .unwrap()
            .contains(&json!("body_unavailable"))
    );
    f.write(
        "main.fpas",
        "program Main; function Leaf(): integer; begin return Missing(); end function; begin end.",
    );
    let incomplete = f.run(
        &["callees", "main.fpas:1:24", "--context", "app.fpasprj"],
        3,
    );
    assert_eq!(incomplete["result"]["direct_edges_complete"], false);
    assert!(incomplete["coverage"]["collections"]["direct_calls"]["known_total"].is_null());
}
