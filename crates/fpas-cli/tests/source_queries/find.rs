//! Name and reference selectors through the isolated CLI worker and schema-2 envelope.
use super::*;

#[test]
fn find_and_name_selectors_share_exact_snapshot_references_and_reject_stale_inputs() {
    let f = Fixture::new("program Main; uses Demo.Math as M; begin discard M.Answer(); end.");
    f.write(
        "app.fpasprj",
        "[project]\nname='app'\nkind='program'\nmain='main.fpas'\n[sources]\ninclude=['*.fpas']\n",
    );
    f.write("math.fpas", "unit Demo.Math; public function Answer(): integer; begin return 42; end function; end unit;");
    let found = f.run(&["find", "Answer", "--context", "app.fpasprj"], 0);
    assert_eq!(found["schema_version"], 2);
    assert_eq!(
        found["request"]["selector"],
        json!({"kind":"search","text":"Answer"})
    );
    assert_eq!(found["limits"]["result_count"], 100);
    let symbols = found["result"]["symbols"].as_array().expect("symbols");
    assert_eq!(symbols.len(), 1);
    assert_eq!(symbols[0]["qualified_name"], "Demo.Math.Answer");
    assert_eq!(symbols[0]["checked"], true);
    let builtin = f.run(&["find", "integer", "--context", "app.fpasprj"], 0);
    let builtin = &builtin["result"]["symbols"][0];
    assert_eq!(builtin["name"], "integer");
    assert_eq!(builtin["origin"], "builtin");
    assert!(builtin["target_ref"].is_null());
    assert_eq!(builtin["reference_unavailable"], "no_source");
    f.write(
        "target.json",
        &serde_json::to_string(&symbols[0]["target_ref"]).unwrap(),
    );
    let definition = f.run(
        &[
            "definition",
            "--symbol",
            "DEMO.math.Answer",
            "--context",
            "app.fpasprj",
        ],
        0,
    );
    assert_eq!(definition["snapshot"], found["snapshot"]);
    assert_eq!(definition["target"]["target_ref"], symbols[0]["target_ref"]);
    let refs = f.run(&["refs", "--target-ref", "target.json"], 0);
    assert_eq!(refs["result"]["locations"].as_array().unwrap().len(), 1);
    assert_eq!(refs["snapshot"], found["snapshot"]);
    let ty = f.run(
        &["type", "--symbol", "Answer", "--context", "app.fpasprj"],
        0,
    );
    assert_eq!(ty["result"]["type"]["kind"], "callable");
    f.write("extra.fpas", "unit Extra; end unit;");
    let stale = f.run(&["definition", "--target-ref", "target.json"], 1);
    assert!(
        stale["diagnostics"]
            .as_array()
            .unwrap()
            .iter()
            .any(|d| d["code"] == "stale_target")
    );
}

#[test]
fn bounded_find_keeps_known_counts_and_ambiguous_selection_never_chooses_first() {
    let f = Fixture::new("program Main; begin end.");
    f.write(
        "app.fpasprj",
        "[project]\nname='app'\nkind='program'\nmain='main.fpas'\n[sources]\ninclude=['*.fpas']\n",
    );
    for name in ["First", "Second"] {
        f.write(&format!("{name}.fpas"), &format!("unit {name}; public function Answer(): integer; begin return 1; end function; end unit;"));
    }
    let found = f.run(
        &["find", "Answer", "--context", "app.fpasprj", "--limit", "1"],
        3,
    );
    assert_eq!(found["coverage"]["analysis_complete"], true);
    assert_eq!(
        found["coverage"]["collections"]["symbols"],
        json!({"returned":1,"known_total":2,"complete":false})
    );
    assert!(found["result"]["symbols"][0]["target_ref"].is_object());
    let ambiguous = f.run(
        &[
            "definition",
            "--symbol",
            "Answer",
            "--context",
            "app.fpasprj",
        ],
        1,
    );
    assert_eq!(ambiguous["outcome"], "error");
    assert!(ambiguous["target"].is_null());
    assert_eq!(ambiguous["candidates"].as_array().unwrap().len(), 2);
    let empty = f.run(&["find", "AbsentValue", "--context", "app.fpasprj"], 0);
    assert_eq!(empty["result"]["symbols"], json!([]));
    for args in [
        vec!["find", "Answer"],
        vec!["definition", "--symbol", "Answer"],
        vec!["find", "", "--context", "app.fpasprj"],
        vec![
            "definition",
            "main.fpas:1:1",
            "--symbol",
            "Main",
            "--context",
            "app.fpasprj",
        ],
    ] {
        let invalid = f.run(&args, 1);
        assert_eq!(invalid["diagnostics"][0]["code"], "invalid_request");
    }
}
