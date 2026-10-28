//! Per-commit observations through real compiled storage and callback paths.

use super::*;
use serde_json::{Value as Json, json};

fn session(source: &str) -> DebugSession {
    let (program, diagnostics) = fpas_parser::parse(source);
    assert!(diagnostics.is_empty(), "{diagnostics:?}");
    DebugSession::new(fpas_compiler::compile(&program).expect("compile trace fixture")).unwrap()
}

fn install(session: &mut DebugSession, selectors: &[&str], limit: usize) {
    session
        .install_trace(
            &selectors
                .iter()
                .map(|name| (*name).into())
                .collect::<Vec<_>>(),
            limit,
        )
        .unwrap();
}

fn report(session: &mut DebugSession) -> Json {
    serde_json::to_value(session.take_trace().expect("trace history")).unwrap()
}

fn values(report: &Json) -> Vec<Json> {
    report["events"]
        .as_array()
        .unwrap()
        .iter()
        .map(|event| event["new_value"]["scalar"].clone())
        .collect()
}

#[test]
fn consecutive_commits_include_initialization_and_never_collapse_backtracking() {
    let mut s = session(
        "program T; var Total: integer := 1; begin Total := 2; Total := 2; Total := 1; end.",
    );
    install(&mut s, &["Total", "TOTAL"], 10);
    assert!(matches!(
        s.continue_execution().unwrap(),
        DebugRunResult::Terminated(_)
    ));
    let r = report(&mut s);
    assert_eq!(r["watches"].as_array().unwrap().len(), 1);
    assert_eq!(values(&r), vec![json!("1"), json!("2"), json!("1")]);
    assert_eq!(r["events"][0]["old_value"]["status"], "uninitialized");
    assert_eq!(r["events"][1]["old_value"]["scalar"], "1");
    assert_eq!(r["events"][2]["old_value"]["scalar"], "2");
    assert_eq!(r["events"][2]["sequence"], 3);
    assert_eq!(r["total_count"], 3);
    assert_eq!(r["complete"], true);
}

#[test]
fn record_descendants_follow_root_replacement_and_reference_commits() {
    let mut s = session(
        r#"program T;
type State = record Count: integer; Other: integer; end record;
var Current: State := State(Count := 1, Other := 2);
procedure SetValue(var N: integer); begin N := 3; N := 4; end procedure;
begin
  Current.Other := 99;
  SetValue(var Current.Count);
  Current := State(Count := 7, Other := 8);
end."#,
    );
    install(&mut s, &["Current.Count", "Current.Other"], 20);
    s.continue_execution().unwrap();
    let r = report(&mut s);
    assert_eq!(
        values(&r),
        vec![
            json!("1"),
            json!("2"),
            json!("99"),
            json!("3"),
            json!("4"),
            json!("7"),
            json!("8")
        ]
    );
    assert_eq!(r["events"][3]["watch_id"], 0);
    assert_eq!(
        r["events"][3]["function"]
            .as_str()
            .unwrap()
            .to_ascii_lowercase(),
        "setvalue"
    );
    assert_eq!(r["events"][3]["location"]["line"], 4);
    assert_eq!(r["events"][3]["task_id"], 0);
}

#[test]
fn synchronous_callbacks_retain_the_actual_writing_function_and_line() {
    let mut s = session(
        r#"program T;
var Total: integer := 0;
function Key(N: integer): integer;
begin
  Total := Total + 1;
  return N;
end function;
begin
  const Items: array of integer := [3, 2, 1];
  discard Items.SortBy(Key);
end."#,
    );
    install(&mut s, &["Total"], 20);
    s.continue_execution().unwrap();
    let r = report(&mut s);
    assert_eq!(
        values(&r),
        vec![json!("0"), json!("1"), json!("2"), json!("3")]
    );
    for event in r["events"].as_array().unwrap().iter().skip(1) {
        assert_eq!(
            event["function"].as_str().unwrap().to_ascii_lowercase(),
            "key"
        );
        assert_eq!(event["location"]["line"], 5);
    }
}

#[test]
fn bad_bindings_are_atomic_and_before_execution() {
    let mut s = session(
        "program T; const Fixed: integer := 1; var Total: integer := 0; begin Total := 2; end.",
    );
    for selectors in [
        vec!["Fixed"],
        vec!["Total", "Missing"],
        vec!["Total()"],
        vec!["Total[0]"],
        vec!["Total + 1"],
    ] {
        assert!(
            s.install_trace(
                &selectors
                    .iter()
                    .map(|name| (*name).into())
                    .collect::<Vec<_>>(),
                10
            )
            .is_err()
        );
        assert!(s.take_trace().is_none());
    }
    install(&mut s, &["Total"], 10);
    s.continue_execution().unwrap();
    assert_eq!(values(&report(&mut s)), vec![json!("0"), json!("2")]);
    assert!(s.install_trace(&["Total".into()], 10).is_err());
}

#[test]
fn event_capacity_succeeds_at_exact_count_and_stops_after_next_committed_change() {
    let source = "program T; var Total: integer := 0; begin Total := 1; Total := 2; end.";
    let mut exact = session(source);
    install(&mut exact, &["Total"], 3);
    exact.continue_execution().unwrap();
    assert_eq!(report(&mut exact)["complete"], true);
    let mut bounded = session(source);
    install(&mut bounded, &["Total"], 2);
    assert_eq!(
        bounded.continue_execution().unwrap_err().kind,
        DebugErrorKind::TraceEventLimit
    );
    let r = report(&mut bounded);
    assert_eq!(values(&r), vec![json!("0"), json!("1")]);
    assert_eq!(r["complete"], false);
    assert_eq!(r["total_count"], 3);
    assert_eq!(r["reasons"], json!(["trace_event_limit"]));
}

#[test]
fn callback_limit_stops_before_following_callback_or_program_writes() {
    let mut s = session(
        r#"program T;
var Total: integer := 0;
var After: integer := 0;
function Key(N: integer): integer;
begin Total := Total + 1; After := 99; return N; end function;
begin const Items: array of integer := [3,2,1]; discard Items.SortBy(Key); After := 100; end."#,
    );
    install(&mut s, &["Total", "After"], 2);
    assert_eq!(
        s.continue_execution().unwrap_err().kind,
        DebugErrorKind::TraceEventLimit
    );
    let r = report(&mut s);
    assert_eq!(values(&r), vec![json!("0"), json!("0")]);
    let snapshot = s.capture_snapshot(&[DebugExpression::Name("After".into())], None, None);
    assert_eq!(
        snapshot.expressions[0].as_ref().unwrap().scalar,
        Some(crate::execution::FailureScalar::Integer("0".into()))
    );
}

#[test]
fn exact_string_comparison_sees_changes_beyond_rendered_prefix() {
    let mut s = session(
        "program T; var Text: string := 'x'.RepeatStr(5000) + 'a'; begin Text := 'x'.RepeatStr(5000) + 'b'; end.",
    );
    install(&mut s, &["Text"], 10);
    s.continue_execution().unwrap();
    let r = report(&mut s);
    assert_eq!(r["event_count"], 2);
    assert_eq!(r["events"][1]["kind"], "change");
    assert_eq!(r["events"][0]["new_value"]["status"], "truncated");
    assert_eq!(r["complete"], true);
    assert_eq!(r["reasons"], json!(["value_truncated"]));
}

#[test]
fn oversized_string_does_not_become_an_empty_successful_trace() {
    let mut s = session("program T; var Text: string := 'x'.RepeatStr(65537); begin null; end.");
    install(&mut s, &["Text"], 10);
    assert_eq!(
        s.continue_execution().unwrap_err().kind,
        DebugErrorKind::TraceValueLimit
    );
    let r = report(&mut s);
    assert_eq!(r["complete"], false);
    assert!(r["total_count"].is_null());
    assert_eq!(r["reasons"], json!(["trace_value_limit"]));
}

#[test]
fn trace_does_not_consume_stopping_data_breakpoint_markers() {
    let mut s = session("program T; var Total: integer := 0; begin Total := 1; Total := 2; end.");
    install(&mut s, &["Total"], 10);
    s.replace_data_breakpoints(vec![DataBreakpoint {
        identity: DebugDataLocationIdentity::Global { index: 0 },
        access: DataBreakpointAccess::Write,
    }])
    .unwrap();
    let stop = stopped(s.continue_execution().unwrap());
    assert_eq!(stop.reason, DebugStopReason::DataBreakpoint);
    let r = report(&mut s);
    assert_eq!(values(&r), vec![json!("0")]);
}

#[test]
fn untraced_sessions_do_not_allocate_a_history() {
    let mut s = session("program T; var Total: integer := 0; begin Total := 1; end.");
    s.continue_execution().unwrap();
    assert!(s.take_trace().is_none());
}

#[test]
fn nominal_scalar_source_names_survive_global_and_record_paths() {
    let mut s = session(
        r#"program T;
type UserId = distinct integer;
type Percent = 0..100;
type Shade = enum Red; Blue; end enum;
type Info = record Id: UserId; Level: Percent; Color: Shade; end record;
var Current: Info := Info(Id := UserId(1), Level := Percent(10), Color := Shade.Red);
begin Current := Info(Id := UserId(2), Level := Percent(20), Color := Shade.Blue); end."#,
    );
    install(
        &mut s,
        &["Current.Id", "Current.Level", "Current.Color"],
        20,
    );
    s.continue_execution().unwrap();
    let r = report(&mut s);
    let types = ["userid", "percent", "shade"];
    for (index, event) in r["events"].as_array().unwrap().iter().enumerate() {
        assert_eq!(
            event["new_value"]["type"]
                .as_str()
                .unwrap()
                .to_ascii_lowercase(),
            types[index % 3]
        );
    }
    assert_eq!(r["event_count"], 6);
}

#[test]
fn equal_signed_zeros_do_not_create_a_change() {
    let mut s =
        session("program T; var Number: real := 0.0; begin Number := -0.0; Number := 1.0; end.");
    install(&mut s, &["Number"], 10);
    s.continue_execution().unwrap();
    assert_eq!(report(&mut s)["event_count"], 2);
}

#[test]
fn repeated_nan_stores_in_verified_external_images_compare_unequal() {
    let (program, errors) =
        fpas_parser::parse("program T; var Number: real := 42.5; begin Number := Number; end.");
    assert!(errors.is_empty());
    let mut image = fpas_compiler::compile(&program).unwrap().into_unverified();
    for constant in &mut image.constants {
        if let Constant::Real(value) = constant {
            *value = f64::NAN.to_bits();
        }
    }
    let mut s = DebugSession::new(image.verify().unwrap()).unwrap();
    install(&mut s, &["Number"], 10);
    s.continue_execution().unwrap();
    let r = report(&mut s);
    assert_eq!(r["event_count"], 2);
    assert_eq!(r["events"][1]["new_value"]["summary"], "NaN");
}

#[test]
fn duplicate_external_metadata_bindings_are_rejected_without_installation() {
    let (program, errors) =
        fpas_parser::parse("program T; var Total: integer := 0; begin Total := 1; end.");
    assert!(errors.is_empty());
    let mut image = fpas_compiler::compile(&program).unwrap().into_unverified();
    let mut duplicate = image.globals[0].clone();
    duplicate.initializer = None;
    image.globals.extend(std::iter::repeat_n(duplicate, 10000));
    let mut s = DebugSession::new(image.verify().unwrap()).unwrap();
    let error = s.install_trace(&["Total".into()], 10).unwrap_err();
    assert!(error.message.contains("multiple eligible"));
    assert!(s.take_trace().is_none());
}

#[test]
fn missing_function_source_maps_are_rejected_before_trace_admission() {
    let (program, errors) =
        fpas_parser::parse("program T; var Total: integer := 0; begin Total := 1; end.");
    assert!(errors.is_empty());
    let mut image = fpas_compiler::compile(&program).unwrap().into_unverified();
    image.source_map.runs.clear();
    for function in &mut image.functions {
        function.debug.sequence_points.clear();
    }
    assert!(matches!(
        image.verify().unwrap_err().kind,
        crate::ValidationErrorKind::MissingFunctionSource { .. }
    ));
}

#[test]
fn value_limit_is_inclusive_and_comparison_work_is_cumulative() {
    let mut exact =
        session("program T; var Text: string := 'x'.RepeatStr(65536); begin Text := Text; end.");
    install(&mut exact, &["Text"], 10);
    exact.continue_execution().unwrap();
    assert_eq!(report(&mut exact)["event_count"], 1);
    let mut bounded = session(
        "program T; var Text: string := 'x'.RepeatStr(65536); begin for I: integer := 1 to 200 do Text := Text; end for; end.",
    );
    install(&mut bounded, &["Text"], 10);
    assert_eq!(
        bounded.continue_execution().unwrap_err().kind,
        DebugErrorKind::TraceWorkLimit
    );
    let r = report(&mut bounded);
    assert_eq!(r["event_count"], 1);
    assert_eq!(r["complete"], false);
    assert!(r["total_count"].is_null());
}

#[test]
fn complete_escaped_history_stays_inside_its_byte_ceiling() {
    let mut s = session(
        "program T; var Total: integer := 0; begin for I: integer := 1 to 10000 do Total := I; end for; end.",
    );
    install(&mut s, &["Total"], 10000);
    assert_eq!(
        s.continue_execution().unwrap_err().kind,
        DebugErrorKind::TraceByteLimit
    );
    let r = report(&mut s);
    assert!(serde_json::to_vec(&r).unwrap().len() <= crate::execution::MAX_TRACE_BYTES);
    assert!(r["event_count"].as_u64().unwrap() < 10000);
    assert_eq!(r["complete"], false);
}

#[test]
fn failed_store_does_not_publish_a_change() {
    let mut s = session("program T; var Total: integer := 0; begin Total := 10 div 0; end.");
    install(&mut s, &["Total"], 10);
    let stop = stopped(s.continue_execution().unwrap());
    assert_eq!(stop.reason, DebugStopReason::RuntimeError);
    assert_eq!(values(&report(&mut s)), vec![json!("0")]);
}

#[test]
fn forbidden_foreign_global_write_preserves_ownership_and_trace() {
    use crate::execution::vm::{debug::trace::TraceHistory, worker::Worker};
    use std::sync::{Arc, Mutex};
    let (program, errors) =
        fpas_parser::parse("program T; var Total: integer := 0; begin Total := 1; end.");
    assert!(errors.is_empty());
    let image = Arc::new(fpas_compiler::compile(&program).unwrap());
    let history = Arc::new(Mutex::new(
        TraceHistory::new(image.executable(), &["Total".into()], 10).unwrap(),
    ));
    let mut worker = Worker::new(Arc::clone(&image)).unwrap();
    worker.task_id = 7;
    worker.debug_trace = Some(Arc::clone(&history));
    let error = worker.run_retained().unwrap_err();
    assert_eq!(error.code, fpas_diagnostics::codes::RUNTIME_INVALID_TASK);
    assert!(history.lock().unwrap().report.events.is_empty());
}
