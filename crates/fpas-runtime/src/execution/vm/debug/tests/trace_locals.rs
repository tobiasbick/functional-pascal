//! Local declaration lifetimes, calls and aliases through actual compiled programs.

use super::*;
use serde_json::{Value as Json, json};

pub(super) fn session(source: &str, selectors: &[&str]) -> DebugSession {
    let (program, diagnostics) = fpas_parser::parse(source);
    assert!(diagnostics.is_empty(), "{diagnostics:?}");
    let image = fpas_compiler::compile(&program).unwrap();
    let mut session = DebugSession::new(image).unwrap();
    session
        .install_trace(
            &selectors.iter().map(|s| s.to_string()).collect::<Vec<_>>(),
            1000,
        )
        .unwrap();
    session
}

pub(super) fn finish(mut s: DebugSession) -> Json {
    let result = s.continue_execution().unwrap();
    assert!(
        matches!(result, DebugRunResult::Terminated(_)),
        "{result:?}"
    );
    serde_json::to_value(s.take_trace().unwrap()).unwrap()
}

pub(super) fn values(report: &Json, watch: usize) -> Vec<Json> {
    report["events"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|event| event["watch_id"] == watch)
        .map(|event| event["new_value"]["scalar"].clone())
        .collect()
}

#[test]
fn scalar_local_and_parameter_events_have_separate_call_and_binding_instances() {
    let r = finish(session(
        "program T; function F(N: integer): integer; begin var Total: integer := N; Total := Total + 1; return Total; end function; begin discard F(1); discard F(4); end.",
        &["F::Total", "f::N"],
    ));
    assert_eq!(
        values(&r, 0),
        vec![json!("1"), json!("2"), json!("4"), json!("5")]
    );
    assert_eq!(values(&r, 1), vec![json!("1"), json!("4")]);
    let events = r["events"].as_array().unwrap();
    assert_eq!(events[0]["watch_id"], 1);
    assert_ne!(events[0]["activation_id"], events[3]["activation_id"]);
    assert_eq!(events[1]["instance_id"], events[2]["instance_id"]);
    assert_eq!(r["complete"], true);
}

#[test]
fn recursive_and_tail_recursive_parameters_do_not_share_activations() {
    for body in [
        "return F(N - 1);",
        "const Next := F(N - 1); return Next + 1;",
    ] {
        let source = format!(
            "program T; function F(N: integer): integer; begin if N = 0 then return 0; end if; {body} end function; begin discard F(3); end."
        );
        let r = finish(session(&source, &["F::N"]));
        assert_eq!(
            values(&r, 0),
            vec![json!("3"), json!("2"), json!("1"), json!("0")]
        );
        let ids: std::collections::BTreeSet<_> = r["events"]
            .as_array()
            .unwrap()
            .iter()
            .map(|e| e["activation_id"].as_str().unwrap())
            .collect();
        assert_eq!(ids.len(), 4);
    }
}

#[test]
fn repeated_lexical_declarations_initialize_new_instances_in_one_call() {
    let r = finish(session(
        "program T; procedure Loop(); begin for I: integer := 1 to 3 do var Item: integer := I; Item := Item + 1; end for; end procedure; begin Loop(); end.",
        &["Loop::Item"],
    ));
    assert_eq!(
        values(&r, 0),
        vec![
            json!("1"),
            json!("2"),
            json!("2"),
            json!("3"),
            json!("3"),
            json!("4")
        ]
    );
    let events = r["events"].as_array().unwrap();
    assert_eq!(events[0]["activation_id"], events[4]["activation_id"]);
    assert_ne!(events[0]["instance_id"], events[2]["instance_id"]);
    assert_eq!(events[2]["kind"], "initialize");
}

#[test]
fn var_aliases_and_their_local_owner_follow_each_commit_in_request_order() {
    let r = finish(session(
        "program T; procedure Inner(var B: integer); begin B := 2; end procedure; procedure Change(var A: integer); begin A := 1; Inner(var A); A := 1; end procedure; procedure Caller(); begin var X: integer := 0; Change(var X); end procedure; begin Caller(); end.",
        &["Inner::B", "Caller::X", "Change::A"],
    ));
    assert_eq!(values(&r, 0), vec![json!("1"), json!("2")]);
    for id in 1..3 {
        assert_eq!(
            values(&r, id),
            vec![json!("0"), json!("1"), json!("2"), json!("1")]
        );
    }
    let changes: Vec<_> = r["events"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|e| e["kind"] == "change")
        .map(|e| e["watch_id"].as_u64().unwrap())
        .collect();
    assert_eq!(changes, vec![1, 2, 0, 1, 2, 1, 2]);
}

#[test]
fn global_and_var_parameter_aliases_share_commit_order_without_changing_global_identity() {
    let r = finish(session(
        "program T; var Total: integer := 0; procedure Change(var N: integer); begin N := 1; N := 2; N := 1; end procedure; begin Change(var Total); end.",
        &["Change::N", "Total"],
    ));
    let changes: Vec<_> = r["events"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|e| e["kind"] == "change")
        .map(|e| e["watch_id"].as_u64().unwrap())
        .collect();
    assert_eq!(changes, vec![0, 1, 0, 1, 0, 1]);
    assert_eq!(
        r["watches"][1]["identity"],
        json!({"global_slot":0,"field_slots":[]})
    );
}

#[test]
fn synchronous_nested_callbacks_preserve_all_live_outer_local_commits() {
    let r = finish(session(
        "program T; procedure Outer(); begin var Total: integer := 0; discard [1, 2, 1].Map(function(N: integer): integer begin discard [N].Map(function(M: integer): integer begin Total := M; return M; end function); return N; end function); end procedure; begin Outer(); end.",
        &["Outer::Total"],
    ));
    assert_eq!(
        values(&r, 0),
        vec![json!("0"), json!("1"), json!("2"), json!("1")]
    );
    for e in r["events"].as_array().unwrap().iter().skip(1) {
        assert_ne!(
            e["function"].as_str().unwrap().to_ascii_lowercase(),
            "outer"
        );
    }
}

#[test]
fn task_local_values_and_callback_parameters_use_actual_writing_task_ids() {
    let r = finish(session(
        "program T; uses Std.Tasks as Tasks; function Work(N: integer): integer; begin var Total: integer := N; Total := Total + 1; return Total; end function; begin scope const A := go Work(3); const B := go Work(8); discard Tasks.Wait(A); discard Tasks.Wait(B); end scope; end.",
        &["Work::Total", "Work::N"],
    ));
    let tasks: std::collections::BTreeSet<_> = r["events"]
        .as_array()
        .unwrap()
        .iter()
        .map(|e| e["task_id"].as_u64().unwrap())
        .collect();
    assert_eq!(tasks, [1, 2].into_iter().collect());
    assert_eq!(r["event_count"], 6);
}

#[test]
fn constant_locals_initialize_and_readonly_strings_do_not_consume_scan_budget() {
    let r = finish(session(
        "program T; function F(Text: string): integer; begin const Answer: integer := 42; var N: integer := 0; while N < 1000 do N := N + 1; end while; return Answer; end function; begin discard F('x'.RepeatStr(65536)); end.",
        &["F::Text", "F::Answer"],
    ));
    assert_eq!(r["event_count"], 2);
    assert_eq!(r["complete"], true);
}

#[test]
fn bad_local_requests_reject_atomically_before_execution() {
    let (program, errors) = fpas_parser::parse(
        "program T; procedure F(); begin var X: integer := 0; begin var X: integer := 1; end; end procedure; begin F(); end.",
    );
    assert!(errors.is_empty());
    let image = fpas_compiler::compile(&program).unwrap();
    for target in ["F::X", "F::Missing", "Missing::X", "F::X[0]", "F::X.Bad"] {
        let mut s = DebugSession::new(image.clone()).unwrap();
        assert!(s.install_trace(&[target.into()], 100).is_err(), "{target}");
        assert!(s.take_trace().is_none());
    }
}

#[test]
fn escaped_captures_stop_tracing_after_the_declaring_scope_ends() {
    let r = finish(session(
        "program T; procedure Outer(); begin var Saved: function(): integer := function(): integer begin return 0; end function; begin var Total: integer := 0; Saved := function(): integer begin Total := Total + 1; return Total; end function; discard Saved(); end; discard Saved(); end procedure; begin Outer(); end.",
        &["Outer::Total"],
    ));
    assert_eq!(values(&r, 0), vec![json!("0"), json!("1")]);
}

#[test]
fn live_instance_and_string_retention_bounds_stop_explicitly() {
    for (source, target) in [
        (
            "program T; function F(N: integer): integer; begin if N = 0 then return 0; end if; const Next := F(N - 1); return Next + 1; end function; begin discard F(1100); end.",
            "F::N",
        ),
        (
            "program T; function F(N: integer; Text: string): integer; begin if N = 0 then return 0; end if; const Next := F(N - 1, Text); return Next + 1; end function; begin discard F(200, 'x'.RepeatStr(65536)); end.",
            "F::Text",
        ),
    ] {
        let mut s = session(source, &[target]);
        s.install_trace(&[target.into()], 10000).unwrap();
        assert_eq!(
            s.continue_execution().unwrap_err().kind,
            DebugErrorKind::TraceValueLimit
        );
        let r = serde_json::to_value(s.take_trace().unwrap()).unwrap();
        assert_eq!(r["complete"], false);
        assert!(r["total_count"].is_null());
    }
}

#[test]
fn completed_calls_release_live_instance_capacity() {
    let mut s = session(
        "program T; function F(N: integer): integer; begin return N; end function; begin for I: integer := 1 to 1100 do discard F(I); end for; end.",
        &["F::N"],
    );
    s.install_trace(&["F::N".into()], 10000).unwrap();
    let r = finish(s);
    assert_eq!(r["event_count"], 1100);
    assert_eq!(r["complete"], true);
}

#[test]
fn inline_main_callbacks_admit_fresh_parameters_and_local_instances() {
    let r = finish(session(
        "program T; function Map(N: integer): integer; begin var Total: integer := N; Total := Total + 1; return Total; end function; begin discard [1, 2, 1].Map(Map); end.",
        &["Map::Total", "Map::N"],
    ));
    assert_eq!(r["event_count"], 9);
    assert_eq!(values(&r, 1), vec![json!("1"), json!("2"), json!("1")]);
    let ids: std::collections::BTreeSet<_> = r["events"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|e| e["watch_id"] == 1)
        .map(|e| e["activation_id"].as_str().unwrap())
        .collect();
    assert_eq!(ids.len(), 3);
}

#[test]
fn external_nonfinite_local_values_end_with_unavailable_coverage() {
    let (program, errors) = fpas_parser::parse(
        "program T; procedure F(N: real); begin const Number: real := N; end procedure; begin F(42.5); end.",
    );
    assert!(errors.is_empty());
    let mut image = fpas_compiler::compile(&program).unwrap().into_unverified();
    for constant in &mut image.constants {
        if let Constant::Real(value) = constant {
            *value = f64::NAN.to_bits();
        }
    }
    for target in ["F::N", "F::Number"] {
        let mut s = DebugSession::new(image.clone().verify().unwrap()).unwrap();
        s.install_trace(&[target.into()], 100).unwrap();
        assert_eq!(
            s.continue_execution().unwrap_err().kind,
            DebugErrorKind::TraceValueLimit
        );
        let r = serde_json::to_value(s.take_trace().unwrap()).unwrap();
        assert_eq!(r["complete"], false);
        assert!(r["total_count"].is_null());
        assert_eq!(r["event_count"], 0);
    }
}

#[test]
fn resumable_callback_operations_keep_their_callers_scope_until_the_last_commit() {
    for (body, expected, writer) in [
        (
            "Total := [1, 2].Reduce(0, Add);",
            vec![json!("0"), json!("3")],
            "work",
        ),
        (
            "discard [1, 2, 3].Map(function(N: integer): integer begin Total := N; return N; end function);",
            vec![json!("0"), json!("1"), json!("2"), json!("3")],
            "",
        ),
    ] {
        let source = format!(
            "program T;\nuses Std.Tasks as Tasks;\nfunction Add(A: integer; B: integer): integer;\nbegin\n return A + B;\nend function;\nfunction Work(): integer;\nbegin\n begin\n var Total: integer := 0;\n {body}\n end;\n return 0;\nend function;\nbegin scope const Job := go Work(); discard Tasks.Wait(Job); end scope; end."
        );
        let r = finish(session(&source, &["Work::Total"]));
        assert_eq!(values(&r, 0), expected);
        assert_eq!(r["complete"], true);
        for event in r["events"].as_array().unwrap().iter().skip(1) {
            assert_eq!(event["location"]["line"], 11);
            assert_eq!(event["task_id"], 1);
            if !writer.is_empty() {
                assert_eq!(event["function"], writer);
            } else {
                assert_ne!(event["function"], "work");
            }
        }
    }
}

#[test]
fn suspended_results_commit_before_scope_exit() {
    for (declarations, statements) in [
        (
            "var Total: integer := 0;",
            "scope const Handle := go Job(); Total := Tasks.Wait(Handle); end scope;",
        ),
        (
            "",
            "scope const Handle := go Job(); begin var Total: integer := 0; Total := Tasks.Wait(Handle); end; end scope;",
        ),
    ] {
        let source = format!(
            "program T; uses Std.Tasks as Tasks, Std.Time as Time; function Job(): integer; begin Time.Sleep(10); return 7; end function; procedure Outer(); begin {declarations} {statements} end procedure; begin Outer(); end."
        );
        let (program, diagnostics) = fpas_parser::parse(&source);
        assert!(diagnostics.is_empty());
        let mut s = DebugSession::with_limits_and_schedule(
            fpas_compiler::compile(&program).unwrap(),
            Vec::new(),
            DebugInspectionLimits::default(),
            DebugExecutionLimits::default(),
            Some(42),
        )
        .unwrap();
        s.install_trace(&["Outer::Total".into()], 1000).unwrap();
        let r = finish(s);
        assert_eq!(values(&r, 0), vec![json!("0"), json!("7")]);
        assert_eq!(r["complete"], true);
    }
}

#[test]
fn exact_event_capacity_and_failed_local_store_preserve_prefix() {
    let source = "program T; procedure F(N: integer); begin var Total: integer := N; Total := Total + 1; Total := 1 div 0; end procedure; begin F(1); end.";
    let mut s = session(source, &["F::N", "F::Total"]);
    s.install_trace(&["F::N".into(), "F::Total".into()], 2)
        .unwrap();
    assert_eq!(
        s.continue_execution().unwrap_err().kind,
        DebugErrorKind::TraceEventLimit
    );
    let r = serde_json::to_value(s.take_trace().unwrap()).unwrap();
    assert_eq!(r["event_count"], 2);
    assert_eq!(r["total_count"], 3);
    let mut s = session(source, &["F::Total"]);
    let stop = stopped(s.continue_execution().unwrap());
    assert_eq!(stop.reason, DebugStopReason::RuntimeError);
    let r = serde_json::to_value(s.take_trace().unwrap()).unwrap();
    assert_eq!(values(&r, 0), vec![json!("1"), json!("2")]);
}
