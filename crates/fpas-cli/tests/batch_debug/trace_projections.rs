//! Concrete debug paths stay distinct from erased runtime record and array layouts.

use super::*;
use serde_json::json;

#[test]
fn imported_intrinsic_array_erasure_reconciles_with_the_defining_unit_layout() {
    let f = Fixture::new(
        "program Erasure; uses Hidden as H; procedure Check(); begin const Value := H.Make(); if Value.Values.Length() <> 1 then panic('bad state'); end if; end procedure; begin Check(); end.",
    );
    f.write(
        "Hidden.fpas",
        "unit Hidden; uses Std.Console as C; public type State = record public Values: array of C.KeyKind; end record; public function Make(): State; begin return State(Values := [C.KeyKind.Character]); end function; end unit;",
    );
    f.write(
        "app.fpasprj",
        "[project]\nname='app'\nkind='program'\nmain='main.fpas'\n[sources]\ninclude=['Hidden.fpas','main.fpas']\n",
    );
    assert_eq!(f.run(&["debug", "app.fpasprj"], 0)["outcome"], "completed");
    let build = f.command().args(["build", "app.fpasprj"]).output().unwrap();
    assert!(
        build.status.success(),
        "{}",
        String::from_utf8_lossy(&build.stderr)
    );
    assert_eq!(
        f.run(&["debug", "app.fpascp", "--source-root", "."], 0)["outcome"],
        "completed"
    );
    for target in ["app.fpasprj", "app.fpascp"] {
        let mut args = vec!["debug", target, "--trace", "Check::Value.Values[0]"];
        if target.ends_with(".fpascp") {
            args.extend(["--source-root", "."]);
        }
        let report = f.run(&args, 1);
        assert_eq!(report["trace"]["reasons"], json!(["binding_failed"]));
        assert!(report["trace"]["events"].as_array().unwrap().is_empty());
    }
}

#[test]
fn mixed_case_payload_bindings_keep_exact_types_and_capture_projections() {
    let f = Fixture::new(
        "program T; type Dollars = distinct integer; type Box of T = record Item: T; end record; type Pair = enum Both(Money: Box of Dollars; Flag: boolean); Empty; end enum; function Make(Choice: Pair): function(): integer; begin case Choice of when Pair.Both(const Amount, const Enabled): return function(): integer begin if Enabled then return integer(Amount.Item); end if; return 0; end function; when Pair.Empty: return function(): integer begin return 0; end function; end case; end function; begin const Read := Make(Pair.Both(Box(Item := Dollars(9)), true)); if Read() <> 9 then panic('wrong mixed payload'); end if; end.",
    );
    let report = trace(
        &f,
        "main.fpas",
        &["Make.$closure_1::Amount.Item", "Make.$closure_1::Enabled"],
    );
    types(&report, &["Dollars", "boolean"]);
    assert_eq!(values(&report, 0), vec![json!("9")]);
    assert_eq!(values(&report, 1), vec![json!(true)]);
}

#[test]
fn bounded_recursive_expansion_preserves_later_nominal_scalar_siblings() {
    let f = Fixture::new(
        "program T; type Dollars = distinct integer; type Node = record Left: array of Node; Right: array of Node; Amount: Dollars; end record; var Root: Node := Node(Left := [], Right := [], Amount := Dollars(1)); begin Root.Amount := Dollars(2); end.",
    );
    let report = trace(
        &f,
        "main.fpas",
        &["Root.Amount", "Root.Left[0].Amount", "Root.Right[0].Amount"],
    );
    types(&report, &["Dollars", "Dollars", "Dollars"]);
    assert_eq!(values(&report, 0), vec![json!("1"), json!("2")]);
    for watch in [1, 2] {
        let events: Vec<_> = report["trace"]["events"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|event| event["watch_id"] == watch)
            .collect();
        assert_eq!(events.len(), 1);
        assert_eq!(events[0]["new_value"]["status"], "unavailable");
        assert_eq!(events[0]["new_value"]["reason"], "index_out_of_bounds");
    }
}

#[test]
fn bounded_wide_record_projection_preserves_legacy_scalar_binding() {
    let mut source =
        String::from("program T; type Dollars = distinct integer; type Wide = record ");
    for index in 0..4097 {
        source.push_str(&format!("F{index}: Dollars; "));
    }
    source.push_str(
        "end record; procedure Check(Value: Wide); begin null; end procedure; begin null; end.",
    );
    let f = Fixture::new(&source);
    let report = trace(&f, "main.fpas", &["Check::Value.F4096"]);
    types(&report, &["Dollars"]);
    assert!(report["trace"]["events"].as_array().unwrap().is_empty());
    assert_eq!(
        report["trace"]["watches"][0]["identity"]["field_slots"],
        json!([4096])
    );
}

fn trace(f: &Fixture, target: &str, selectors: &[&str]) -> Value {
    let mut args = vec!["debug", target];
    if target.ends_with(".fpascp") {
        args.extend(["--source-root", "."]);
    }
    for selector in selectors {
        args.extend(["--trace", selector]);
    }
    f.run(&args, 0)
}

fn values(report: &Value, watch: usize) -> Vec<Value> {
    report["trace"]["events"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|event| event["watch_id"] == watch)
        .map(|event| event["new_value"]["scalar"].clone())
        .collect()
}

fn types(report: &Value, expected: &[&str]) {
    assert_eq!(
        report["trace"]["watches"].as_array().unwrap().len(),
        expected.len()
    );
    for (watch, expected) in expected.iter().enumerate() {
        assert_eq!(report["trace"]["watches"][watch]["type"], *expected);
        for event in report["trace"]["events"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|event| event["watch_id"] == watch)
        {
            assert_eq!(event["old_value"]["type"], *expected);
            assert_eq!(event["new_value"]["type"], *expected);
        }
    }
    assert_eq!(report["trace"]["complete"], true);
}

#[test]
fn shared_generic_layouts_keep_applied_root_types_across_source_project_and_image() {
    let f = Fixture::new(
        "program T; type Dollars = distinct integer; type Box of T = record Item: T; end record; var Number: Box of integer := Box(Item := 1); var Text: Box of string := Box(Item := 'one'); var Cash: Box of Dollars := Box(Item := Dollars(5)); procedure Inspect(N: Box of integer; S: Box of string; D: Box of Dollars); begin var Explicit: Box of integer := N; var Inferred := Box(Item := 'two'); const Fixed := D; Explicit.Item := 2; Inferred.Item := 'three'; end procedure; begin Inspect(Number, Text, Cash); Number.Item := 3; Text.Item := 'four'; Cash.Item := Dollars(6); end.",
    );
    f.write("app.fpasprj", "[project]\nname='app'\nkind='program'\nmain='main.fpas'\n[sources]\ninclude=['main.fpas']\n");
    let selectors = [
        "Number.Item",
        "Text.Item",
        "Cash.Item",
        "Inspect::N.Item",
        "Inspect::S.Item",
        "Inspect::D.Item",
        "Inspect::Explicit.Item",
        "Inspect::Inferred.Item",
        "Inspect::Fixed.Item",
    ];
    let source = trace(&f, "main.fpas", &selectors);
    types(
        &source,
        &[
            "integer", "string", "Dollars", "integer", "string", "Dollars", "integer", "string",
            "Dollars",
        ],
    );
    for (watch, expected) in [
        vec![json!("1"), json!("3")],
        vec![json!("one"), json!("four")],
        vec![json!("5"), json!("6")],
        vec![json!("1")],
        vec![json!("one")],
        vec![json!("5")],
        vec![json!("1"), json!("2")],
        vec![json!("two"), json!("three")],
        vec![json!("5")],
    ]
    .into_iter()
    .enumerate()
    {
        assert_eq!(values(&source, watch), expected);
    }
    assert_eq!(
        source["trace"],
        trace(&f, "app.fpasprj", &selectors)["trace"]
    );
    let build = f.command().args(["build", "app.fpasprj"]).output().unwrap();
    assert!(
        build.status.success(),
        "{}",
        String::from_utf8_lossy(&build.stderr)
    );
    assert_eq!(
        source["trace"],
        trace(&f, "app.fpascp", &selectors)["trace"]
    );
}

#[test]
fn generic_var_aliases_keep_nominal_projection_after_field_and_record_replacement() {
    let f = Fixture::new(
        "program T; type Dollars = distinct integer; type Box of T = record Item: T; end record; var State: Box of Dollars := Box(Item := Dollars(1)); procedure Change(var Alias: Box of Dollars); begin Alias.Item := Dollars(2); Alias := Box(Item := Dollars(3)); end procedure; begin Change(var State); end.",
    );
    let report = trace(&f, "main.fpas", &["State.Item", "Change::Alias.Item"]);
    types(&report, &["Dollars", "Dollars"]);
    for watch in 0..2 {
        assert_eq!(
            values(&report, watch),
            vec![json!("1"), json!("2"), json!("3")]
        );
    }
    let writes: Vec<_> = report["trace"]["events"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|event| event["kind"] == "change")
        .map(|event| event["watch_id"].clone())
        .collect();
    assert_eq!(writes, vec![json!(0), json!(1), json!(0), json!(1)]);
}

#[test]
fn nested_alias_generic_and_recursive_records_preserve_absent_index_types() {
    let f = Fixture::new(
        "program T; type Dollars = distinct integer; type Box of T = record Item: T; end record; type CashBox = Box of Dollars; type Rows = array of CashBox; type Envelope of T = record Items: array of Box of T; end record; type Node of T = record Item: T; Next: Option of Node of T := None; end record; procedure Work(Input: Envelope of Dollars); begin var State := Input; const Nested: Box of Rows := Box(Item := [Box(Item := Dollars(7))]); const Root: Node of Dollars := Node(Item := Dollars(11)); State := Envelope(Items := [Box(Item := Dollars(8))]); State.Items := []; State.Items := [Box(Item := Dollars(9))]; end procedure; begin Work(Envelope(Items := [Box(Item := Dollars(5))])); end.",
    );
    let report = trace(
        &f,
        "main.fpas",
        &[
            "Work::State.Items[0].Item",
            "Work::State.Items[7].Item",
            "Work::Nested.Item[0].Item",
            "Work::Root.Item",
        ],
    );
    types(&report, &["Dollars", "Dollars", "Dollars", "Dollars"]);
    assert_eq!(
        values(&report, 0),
        vec![json!("5"), json!("8"), json!(null), json!("9")]
    );
    assert_eq!(values(&report, 1), vec![json!(null)]);
    assert_eq!(values(&report, 2), vec![json!("7")]);
    assert_eq!(values(&report, 3), vec![json!("11")]);
    let absence = report["trace"]["events"]
        .as_array()
        .unwrap()
        .iter()
        .find(|event| event["watch_id"] == 1)
        .unwrap();
    assert_eq!(absence["new_value"]["reason"], "index_out_of_bounds");
}

#[test]
fn nominal_array_elements_retain_distinct_subrange_and_enum_names_at_every_depth() {
    let f = Fixture::new(
        "program T; type Dollars = distinct integer; type Limit = 1..10; type Mood = enum Sad; Glad; end enum; type DollarList = array of Dollars; type Box of T = record Item: T; end record; var Global: DollarList := [Dollars(4)]; procedure Inspect(Cash: DollarList; Small: array of Limit; Moods: array of Mood); begin const Nested: array of array of Dollars := [Cash]; const Bounded: Box of array of Limit := Box(Item := Small); const Feelings := Moods; end procedure; begin Inspect(Global, [Limit(3)], [Mood.Glad]); end.",
    );
    let report = trace(
        &f,
        "main.fpas",
        &[
            "Global[0]",
            "Inspect::Cash[0]",
            "Inspect::Small[0]",
            "Inspect::Moods[0]",
            "Inspect::Nested[0][0]",
            "Inspect::Bounded.Item[0]",
            "Inspect::Feelings[0]",
        ],
    );
    types(
        &report,
        &[
            "Dollars", "Dollars", "Limit", "Mood", "Dollars", "Limit", "Mood",
        ],
    );
    assert_eq!(values(&report, 0), vec![json!("4")]);
    assert_eq!(values(&report, 2), vec![json!("3")]);
    assert_eq!(values(&report, 4), vec![json!("4")]);
}

#[test]
fn immutable_transitive_captures_keep_applied_types_and_exact_source_provenance() {
    let f = Fixture::new(
        "program T; type Dollars = distinct integer; type Box of T = record Item: T; end record; function Make(State: Box of Dollars): function(): integer; function Middle(): function(): integer; function Read(): integer; begin return integer(State.Item); end function; begin return Read; end function; begin return Middle(); end function; begin const Read := Make(Box(Item := Dollars(7))); discard Read(); discard Read(); end.",
    );
    let report = trace(
        &f,
        "main.fpas",
        &[
            "Make.Middle.Read::State.Item",
            "Make.Middle::State.Item",
            "Make::State.Item",
        ],
    );
    types(&report, &["Dollars", "Dollars", "Dollars"]);
    assert_eq!(values(&report, 0), vec![json!("7"), json!("7")]);
    assert_eq!(values(&report, 1), vec![json!("7")]);
    assert_eq!(values(&report, 2), vec![json!("7")]);
    let watches = &report["trace"]["watches"];
    for capture in 0..2 {
        assert_eq!(watches[capture]["identity"]["kind"], "capture");
        assert_eq!(
            watches[capture]["identity"]["capture_source"]["function_id"],
            watches[2]["identity"]["function_id"]
        );
        assert_eq!(
            watches[capture]["identity"]["capture_source"]["binding_id"],
            watches[2]["identity"]["binding_id"]
        );
    }
}

#[test]
fn mutable_generic_captures_keep_active_histories_and_trace_limits() {
    let f = Fixture::new(
        "program T; type Dollars = distinct integer; type Box of T = record Item: T; end record; function Make(): function(): integer; function Next(): integer; begin State.Item := Dollars(integer(State.Item) + 1); return integer(State.Item); end function; begin var State := Box(Item := Dollars(1)); return Next; end function; begin const Next := Make(); discard Next(); discard Next(); end.",
    );
    let report = trace(&f, "main.fpas", &["Make.Next::State.Item"]);
    types(&report, &["Dollars"]);
    assert_eq!(
        values(&report, 0),
        vec![json!("1"), json!("2"), json!("2"), json!("3")]
    );
    for event in report["trace"]["events"].as_array().unwrap() {
        assert_eq!(event["function"], "make.next");
    }
    let limited = f.run(
        &[
            "debug",
            "main.fpas",
            "--trace",
            "Make.Next::State.Item",
            "--trace-limit",
            "3",
        ],
        3,
    );
    assert_eq!(
        values(&limited, 0),
        vec![json!("1"), json!("2"), json!("2")]
    );
    assert_eq!(limited["trace"]["event_count"], 3);
    assert_eq!(limited["trace"]["total_count"], 4);
    assert_eq!(limited["diagnostics"][0]["code"], "trace_event_limit");
}

#[test]
fn unsupported_generic_paths_reject_atomically_before_initializer_effects() {
    let f = Fixture::new(
        "program T; uses Std.Fs as Fs; type Box of T = record Item: T; end record; function Init(): integer; begin discard Fs.WriteText('executed', 'bad').Unwrap(); return 0; end function; var Seed: integer := Init(); procedure Unknown<T>(Value: T); begin var State := Box(Item := Value); end procedure; function Identity(): integer; begin return 1; end function; procedure Work(); begin const Good := Box(Item := 1); const Optional: Box of Option of integer := Box(Item := Some(1)); const Callback: Box of function(): integer := Box(Item := Identity); const Aggregate := Box(Item := [1]); end procedure; begin Work(); end.",
    );
    for unsupported in [
        "Unknown::State.Item",
        "Work::Good",
        "Work::Optional.Item",
        "Work::Optional.Item.Value",
        "Work::Callback.Item",
        "Work::Aggregate.Item",
        "Work::Good.Item.NoSuch",
    ] {
        let report = f.run(
            &[
                "debug",
                "main.fpas",
                "--trace",
                "Work::Good.Item",
                "--trace",
                unsupported,
            ],
            1,
        );
        assert_eq!(
            report["trace"]["reasons"],
            json!(["binding_failed"]),
            "{unsupported}: {report}"
        );
        assert!(report["trace"]["watches"].as_array().unwrap().is_empty());
        assert!(!f.0.join("executed").exists());
    }
}

#[test]
fn imported_generic_and_recursive_projections_survive_library_object_cache_and_image() {
    let f = Fixture::new(
        "program Main; uses Ledger as L; procedure UseLedger(); begin const Tree: L.Node of L.Dollars := L.Node(Item := L.Dollars(5), Children := [L.Node(Item := L.Dollars(7), Children := [])]); const Read := L.Make(L.Money); L.Change(var L.Money); discard Read(); var Local: L.Box of L.Dollars := L.Money; Local.Item := L.Dollars(3); end procedure; begin UseLedger(); end.",
    );
    f.write(
        "lib/Ledger.fpas",
        "unit Ledger; public type Dollars = distinct integer; public type Box of T = record public Item: T; end record; public type Node of T = record public Item: T; public Children: array of Node of T; end record; public var Money: Box of Dollars := Box(Item := Dollars(1)); public function Make(Value: Box of Dollars): function(): integer; function Read(): integer; begin return integer(Value.Item); end function; begin return Read; end function; public procedure Change(var Value: Box of Dollars); begin Value.Item := Dollars(2); end procedure; end unit;",
    );
    f.write(
        "lib/lib.fpasprj",
        "[project]\nname='ledger'\nkind='library'\n[sources]\ninclude=['*.fpas']\n[exports]\nunits=['Ledger']\n",
    );
    f.write(
        "app.fpasprj",
        "[project]\nname='app'\nkind='program'\nmain='main.fpas'\n[sources]\ninclude=['main.fpas']\n[dependencies]\nprojects=['lib/lib.fpasprj']\n",
    );
    let selectors = [
        "Ledger.Money.Item",
        "Ledger.Make::Value.Item",
        "Ledger.Change::Value.Item",
        "Ledger.Make.Read::Value.Item",
        "UseLedger::Local.Item",
        "UseLedger::Tree.Children[0].Item",
        "UseLedger::Tree.Children[0].Children[0].Item",
    ];
    let cold = trace(&f, "app.fpasprj", &selectors);
    types(&cold, &["Dollars"; 7]);
    for (watch, expected) in [
        vec![json!("1"), json!("2")],
        vec![json!("1")],
        vec![json!("1"), json!("2")],
        vec![json!("1")],
        vec![json!("2"), json!("3")],
        vec![json!("7")],
        vec![json!(null)],
    ]
    .into_iter()
    .enumerate()
    {
        assert_eq!(values(&cold, watch), expected);
    }
    let object = f.0.join("lib/Ledger.fpascu");
    let stamp = fs::metadata(&object).unwrap().modified().unwrap();
    assert_eq!(cold["trace"], trace(&f, "app.fpasprj", &selectors)["trace"]);
    assert_eq!(
        stamp,
        fs::metadata(&object).unwrap().modified().unwrap(),
        "the second project run must reuse the compiled library object"
    );
    let build = f.command().args(["build", "app.fpasprj"]).output().unwrap();
    assert!(
        build.status.success(),
        "{}",
        String::from_utf8_lossy(&build.stderr)
    );
    assert_eq!(
        stamp,
        fs::metadata(&object).unwrap().modified().unwrap(),
        "image publication must reuse the same library object"
    );
    assert_eq!(cold["trace"], trace(&f, "app.fpascp", &selectors)["trace"]);
}
