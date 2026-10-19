//! Fixed-position paths, known absence and exact event/lifetime semantics.

use super::trace_locals::{finish, session, values};
use super::*;
use serde_json::json;

#[test]
fn global_index_availability_is_a_known_state_with_exact_changes_and_counts() {
    let r = finish(session(
        "program T; var Items: array of integer := []; begin Items := []; Items := [1]; Items := [1, 2]; Items[1] := 3; Items := []; Items := [4, 3]; end.",
        &["Items[1]", "items[1]", "Items[9223372036854775807]"],
    ));
    assert_eq!(r["watches"].as_array().unwrap().len(), 2);
    assert_eq!(
        values(&r, 0),
        vec![json!(null), json!("2"), json!("3"), json!(null), json!("3")]
    );
    assert_eq!(values(&r, 1), vec![json!(null)]);
    assert_eq!(r["complete"], true);
    assert_eq!(r["total_count"], 6);
    assert_eq!(r["reasons"], json!([]));
    assert_eq!(r["events"][0]["kind"], "initialize");
    assert_eq!(r["events"][0]["new_value"]["status"], "unavailable");
    assert_eq!(r["events"][0]["new_value"]["reason"], "index_out_of_bounds");
    assert_eq!(
        r["watches"][1]["identity"]["path"][0]["index"],
        "9223372036854775807"
    );
}

#[test]
fn mixed_record_array_paths_follow_positions_through_sort_and_replacement() {
    let r = finish(session(
        "program T; type Row = record X: integer; end record; type Model = record Rows: array of Row; end record; procedure F(); begin var State := Model(Rows := [Row(X := 3), Row(X := 1)]); State.Rows := State.Rows.SortBy(function(Item: Row): integer begin return Item.X; end function); State.Rows[0].X := 4; State.Rows := []; State.Rows := State.Rows.Push(Row(X := 5)); end procedure; begin F(); end.",
        &["F::State.Rows[0].X", "F::State.Rows[1].X"],
    ));
    assert_eq!(
        values(&r, 0),
        vec![json!("3"), json!("1"), json!("4"), json!(null), json!("5")]
    );
    assert_eq!(values(&r, 1), vec![json!("1"), json!("3"), json!(null)]);
    assert_eq!(r["events"][0]["instance_id"], r["events"][1]["instance_id"]);
    assert_eq!(
        r["watches"][0]["identity"]["path"],
        json!([
            {"kind":"field","slot":0},{"kind":"index","index":"0"},{"kind":"field","slot":0}
        ])
    );
}

#[test]
fn var_array_global_aliases_keep_request_order_for_mutating_operations() {
    let r = finish(session(
        "program T; var Items: array of integer := [9, 1]; procedure Change(var A: array of integer); begin A[0] := 5; A := A.Push(2); A.SortInPlace(); end procedure; begin Change(var Items); end.",
        &["Change::A[0]", "Items[0]", "Items[2]"],
    ));
    assert_eq!(values(&r, 0), vec![json!("9"), json!("5"), json!("1")]);
    assert_eq!(values(&r, 1), values(&r, 0));
    assert_eq!(values(&r, 2), vec![json!(null), json!("2"), json!("5")]);
    let changes: Vec<_> = r["events"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|e| e["kind"] == "change")
        .map(|e| e["watch_id"].as_u64().unwrap())
        .collect();
    assert_eq!(changes, vec![0, 1, 2, 0, 1, 2]);
}

#[test]
fn captured_array_writes_in_nested_callbacks_keep_position_history_until_scope_exit() {
    let r = finish(session(
        "program T; procedure F(); begin var Items: array of integer := []; discard [1, 2, 1].Map(function(N: integer): integer begin discard [N].Map(function(M: integer): integer begin Items := [M]; return M; end function); return N; end function); end procedure; begin F(); end.",
        &["F::Items[0]"],
    ));
    assert_eq!(
        values(&r, 0),
        vec![json!(null), json!("1"), json!("2"), json!("1")]
    );
    for e in r["events"].as_array().unwrap().iter().skip(1) {
        assert_ne!(e["function"], "f");
    }
}

#[test]
fn absent_paths_still_consume_live_capacity_and_finished_calls_release_it() {
    let mut s = session(
        "program T; function F(N: integer; Input: array of integer): integer; begin if N = 0 then return 0; end if; const Next := F(N - 1, Input); return Next + 1; end function; begin discard F(600, []); end.",
        &["F::Input[0]", "F::Input[1]"],
    );
    s.install_trace(&["F::Input[0]".into(), "F::Input[1]".into()], 10000)
        .unwrap();
    assert_eq!(
        s.continue_execution().unwrap_err().kind,
        DebugErrorKind::TraceValueLimit
    );
    let r = serde_json::to_value(s.take_trace().unwrap()).unwrap();
    assert_eq!(r["complete"], false);
    assert!(r["total_count"].is_null());
    let mut s = session(
        "program T; procedure F(Input: array of integer); begin null; end procedure; begin for I: integer := 1 to 1100 do F([]); end for; end.",
        &["F::Input[0]"],
    );
    s.install_trace(&["F::Input[0]".into()], 10000).unwrap();
    assert_eq!(finish(s)["event_count"], 1100);
}

#[test]
fn unsupported_index_spellings_and_containers_fail_atomically() {
    let source = "program T; procedure F(); begin const Items: array of integer := []; const Text: string := 'x'; end procedure; begin F(); end.";
    let (p, errors) = fpas_parser::parse(source);
    assert!(errors.is_empty());
    let image = fpas_compiler::compile(&p).unwrap();
    for bad in [
        "F::Items[-1]",
        "F::Items[+1]",
        "F::Items[01]",
        "F::Items[ 0]",
        "F::Items[0 ]",
        "F::Items[1+1]",
        "F::Items[N]",
        "F::Items[*]",
        "F::Items[0..1]",
        "F::Items[9223372036854775808]",
        "F::Text[0]",
        "F::Items[0]X",
        "F::Items[]",
        "F::Items[0",
    ] {
        let mut s = DebugSession::new(image.clone()).unwrap();
        assert!(
            s.install_trace(&["F::Items[0]".into(), bad.into()], 10)
                .is_err(),
            "{bad}"
        );
        assert!(s.take_trace().is_none());
    }
}

#[test]
fn failed_index_store_keeps_known_absence_without_creating_a_successful_change() {
    let mut s = session(
        "program T; procedure F(); begin var Items: array of integer := []; Items[0] := 1; end procedure; begin F(); end.",
        &["F::Items[0]"],
    );
    assert_eq!(
        stopped(s.continue_execution().unwrap()).reason,
        DebugStopReason::RuntimeError
    );
    let r = serde_json::to_value(s.take_trace().unwrap()).unwrap();
    assert_eq!(r["event_count"], 1);
    assert_eq!(r["complete"], true);
    assert_eq!(r["events"][0]["new_value"]["reason"], "index_out_of_bounds");
}

#[test]
fn array_depth_bound_is_exact_and_uses_concrete_types_instead_of_display_prefixes() {
    for depth in [16, 17] {
        let mut source = "program T; type L0 = integer;".to_owned();
        let mut value = "1".to_owned();
        for n in 1..=depth {
            source.push_str(&format!("type L{n} = array of L{};", n - 1));
            value = format!("[{value}]");
        }
        source.push_str(&format!(
            "procedure F(Input: L{depth}); begin null; end procedure; begin F({value}); end."
        ));
        let target = format!("F::Input{}", "[0]".repeat(depth));
        let (p, errors) = fpas_parser::parse(&source);
        assert!(errors.is_empty());
        let mut s = DebugSession::new(fpas_compiler::compile(&p).unwrap()).unwrap();
        if depth == 16 {
            s.install_trace(&[target], 10).unwrap();
            let r = finish(s);
            assert_eq!(values(&r, 0), vec![json!("1")]);
            assert_eq!(r["watches"][0]["type"], "integer");
        } else {
            assert!(s.install_trace(&[target], 10).is_err());
        }
    }
}

#[test]
fn indexed_string_comparison_uses_full_values_past_the_presented_prefix() {
    let r = finish(session(
        "program T; procedure F(); begin var Items := ['x'.RepeatStr(6000)]; Items[0] := 'x'.RepeatStr(5999) + 'y'; end procedure; begin F(); end.",
        &["F::Items[0]"],
    ));
    assert_eq!(r["event_count"], 2);
    assert_eq!(r["complete"], true);
    assert_eq!(r["reasons"], json!(["value_truncated"]));
    assert_eq!(
        r["events"][1]["old_value"]["scalar"],
        r["events"][1]["new_value"]["scalar"]
    );
}

#[test]
fn indexed_types_use_machine_metadata_and_do_not_parse_display_names() {
    let (p, errors) = fpas_parser::parse(
        "program T; type Dollars = distinct integer; procedure F(Items: array of Dollars; Flags: array of boolean); begin null; end procedure; begin F([Dollars(1)], [true]); end.",
    );
    assert!(errors.is_empty());
    let mut image = fpas_compiler::compile(&p).unwrap().into_unverified();
    let f = image
        .functions
        .iter_mut()
        .find(|f| image.strings.get(f.name) == Some("f"))
        .unwrap();
    let boolean_display = f.debug.bindings[1].type_name;
    f.debug.bindings[0].type_name = boolean_display;
    let mut s = DebugSession::new(image.verify().unwrap()).unwrap();
    s.install_trace(&["F::Items[0]".into()], 10).unwrap();
    let r = finish(s);
    assert_eq!(r["watches"][0]["type"], "integer");
    assert_eq!(r["events"][0]["new_value"]["type"], "integer");
    assert_eq!(values(&r, 0), vec![json!("1")]);
}
