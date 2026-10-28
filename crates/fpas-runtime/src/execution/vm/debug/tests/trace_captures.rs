//! Activation-scoped capture histories with exact static source provenance.

use super::trace_locals::{finish, session, values};
use super::*;
use serde_json::json;

#[test]
fn escaped_cells_initialize_for_each_closure_activation_without_extending_owner_history() {
    let r = finish(session(
        "program T; function Make(Seed: integer): function(): integer; function Next(): integer; begin Total := Total + 1; return Total; end function; begin var Total := Seed; return Next; end function; begin const A := Make(1); const Copy := A; const B := Make(10); discard A(); discard B(); discard Copy(); end.",
        &["Make.Next::Total", "Make::Total", "make.next::total"],
    ));
    assert_eq!(
        values(&r, 0),
        vec![
            json!("1"),
            json!("2"),
            json!("10"),
            json!("11"),
            json!("2"),
            json!("3")
        ]
    );
    assert_eq!(values(&r, 1), vec![json!("1"), json!("10")]);
    assert_eq!(r["watches"].as_array().unwrap().len(), 2);
    assert_eq!(r["watches"][0]["identity"]["kind"], "capture");
    assert_eq!(
        r["watches"][0]["identity"]["capture_source"]["binding_id"],
        r["watches"][1]["identity"]["binding_id"]
    );
    let ids: std::collections::BTreeSet<_> = r["events"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|e| e["watch_id"] == 0)
        .map(|e| e["activation_id"].as_str().unwrap())
        .collect();
    assert_eq!(ids.len(), 3);
    assert_eq!(r["complete"], true);
}

#[test]
fn immutable_capture_types_come_from_exact_source_declarations_before_abi_reuse() {
    let r = finish(session(
        "program T; type Dollars = distinct integer; type Shade = enum Red; Blue; end enum; function Make(Amount: Dollars; Color: Shade): function(N: integer): integer; function Read(N: integer): integer; begin discard [1, 2].Map(function(M: integer): integer begin return M + N; end function); if Color = Shade.Red then return integer(Amount) + N; end if; return 0; end function; begin return Read; end function; begin const A := Make(Dollars(7), Shade.Red); discard A(1); discard A(2); end.",
        &["Make.Read::Amount", "Make.Read::Color", "Make.Read::N"],
    ));
    assert_eq!(values(&r, 0), vec![json!("7"), json!("7")]);
    assert_eq!(values(&r, 1), vec![json!("0"), json!("0")]);
    assert_eq!(values(&r, 2), vec![json!("1"), json!("2")]);
    assert_eq!(r["watches"][0]["type"], "Dollars");
    assert_eq!(r["watches"][1]["type"], "Shade");
    assert_eq!(r["events"][0]["new_value"]["type"], "Dollars");
    assert_eq!(r["complete"], true);
}

#[test]
fn transitive_enclosing_cells_and_nested_callbacks_keep_writer_and_request_order() {
    let r = finish(session(
        "program T; function Make(): function(): integer; function Mid(): function(): integer; function Next(): integer; begin discard [1, 2].Map(function(N: integer): integer begin Total := Total + N; return N; end function); return Total; end function; begin return Next; end function; begin var Total := 0; return Mid(); end function; begin const A := Make(); discard A(); end.",
        &["Make.Mid.Next::Total", "Make.Mid::Total", "Make::Total"],
    ));
    assert_eq!(values(&r, 0), vec![json!("0"), json!("1"), json!("3")]);
    assert_eq!(values(&r, 1), vec![json!("0")]);
    assert_eq!(values(&r, 2), vec![json!("0")]);
    assert_ne!(r["events"][3]["function"], "make.mid.next");
    assert_eq!(
        r["watches"][0]["identity"]["capture_source"]["function_id"],
        r["watches"][2]["identity"]["function_id"]
    );
    assert_eq!(r["complete"], true);
}

#[test]
fn parameters_and_locals_win_over_transitive_capture_names() {
    let r = finish(session(
        "program T; procedure Outer(var Value: integer); procedure First(); begin Value := Value + 1; end procedure; procedure Second(Value: integer); begin First(); end procedure; procedure Third(); begin const Value := 'local'; Second(100); end procedure; begin Third(); end procedure; begin var Total := 0; Outer(var Total); end.",
        &[
            "Outer.Second::Value",
            "Outer.Third::Value",
            "Outer.First::Value",
            "Outer::Value",
        ],
    ));
    assert_eq!(values(&r, 0), vec![json!("100")]);
    assert_eq!(values(&r, 1), vec![json!("local")]);
    assert_eq!(values(&r, 2), vec![json!("0"), json!("1")]);
    assert_eq!(values(&r, 3), values(&r, 2));
    assert_eq!(r["watches"][0]["identity"]["kind"], "parameter");
    assert_eq!(r["watches"][1]["identity"]["kind"], "local");
    assert_eq!(r["complete"], true);
}

#[test]
fn nearer_ancestor_capture_wins_over_same_named_transitive_capture() {
    let r = finish(session(
        "program T; function Outer(Value: string): function(): integer; function First(): string; begin return Value; end function; function Inner(Value: integer): function(): integer; function Second(): string; begin return First(); end function; function Third(): integer; begin const Copy := Value; discard Second(); return Copy; end function; begin return Third; end function; begin return Inner(42); end function; begin const A := Outer('outer'); discard A(); end.",
        &["Outer.Inner.Third::Value", "Outer.First::Value"],
    ));
    assert_eq!(values(&r, 0), vec![json!("42")]);
    assert_eq!(values(&r, 1), vec![json!("outer")]);
    assert_eq!(r["complete"], true);
}

#[test]
fn unrelated_owner_block_shadow_does_not_hide_a_transitive_capture() {
    let r = finish(session(
        "program T; function Outer(Value: integer): function(): integer; function Mid(): function(): integer; function Read(): integer; begin return Value; end function; begin begin const Value := 'shadow'; discard Value; end; return Read; end function; begin return Mid(); end function; begin const A := Outer(42); discard A(); end.",
        &["Outer.Mid.Read::Value"],
    ));
    assert_eq!(values(&r, 0), vec![json!("42")]);
    assert_eq!(r["complete"], true);
}

#[test]
fn only_transitive_capture_is_rejected_when_a_nearer_parameter_hides_its_name() {
    let (p, errors) = fpas_parser::parse(
        "program T; function Outer(Value: string): function(): string; function First(): string; begin return Value; end function; function Inner(Value: integer): function(): string; function Second(): string; begin return First(); end function; begin return Second; end function; begin return Inner(42); end function; begin const A := Outer('outer'); discard A(); end.",
    );
    assert!(errors.is_empty());
    let mut s = DebugSession::new(fpas_compiler::compile(&p).unwrap()).unwrap();
    assert!(
        s.install_trace(&["Outer.Inner.Second::Value".into()], 10)
            .is_err()
    );
}

#[test]
fn inline_capture_visibility_uses_creation_scope_and_declaration_order() {
    for body in [
        "const Callback := function(): integer begin return Value; end function; const Value := 'shadow'; discard Value; return Callback;",
        "begin const Value := 'shadow'; discard Value; end; return function(): integer begin return Value; end function;",
    ] {
        let source = format!(
            "program T; function Outer(Value: integer): function(): integer; function Mid(): function(): integer; begin {body} end function; begin return Mid(); end function; begin const A := Outer(42); discard A(); end."
        );
        let (p, errors) = fpas_parser::parse(&source);
        assert!(errors.is_empty());
        let image = fpas_compiler::compile(&p).unwrap();
        let anonymous = image
            .executable()
            .functions
            .iter()
            .find(|f| {
                image
                    .executable()
                    .strings
                    .get(f.name)
                    .is_some_and(|n| n.contains("$closure_"))
            })
            .unwrap();
        let selector = format!(
            "{}::Value",
            image.executable().strings.get(anonymous.name).unwrap()
        );
        let mut s = DebugSession::new(image).unwrap();
        s.install_trace(&[selector], 10)
            .unwrap_or_else(|e| panic!("{e:?}: {body}"));
        let r = finish(s);
        assert_eq!(values(&r, 0), vec![json!("42")]);
        assert_eq!(r["complete"], true);
    }
}

#[test]
fn capture_paths_share_one_admission_instance_and_keep_absence_known() {
    let r = finish(session(
        "program T; type Model = record Items: array of integer; Flag: boolean; end record; function Make(): function(): integer; function Next(): integer; begin State := Model(Items := [7], Flag := true); State.Items := []; return 0; end function; begin var State := Model(Items := [], Flag := false); return Next; end function; begin const A := Make(); discard A(); discard A(); end.",
        &["Make.Next::State.Items[0]", "Make.Next::State.Flag"],
    ));
    assert_eq!(
        values(&r, 0),
        vec![
            json!(null),
            json!("7"),
            json!(null),
            json!(null),
            json!("7"),
            json!(null)
        ]
    );
    assert_eq!(values(&r, 1), vec![json!(false), json!(true), json!(true)]);
    let events = r["events"].as_array().unwrap();
    assert_eq!(events[0]["instance_id"], events[1]["instance_id"]);
    assert_eq!(events[2]["watch_id"], 0);
    assert_eq!(events[3]["watch_id"], 1);
    assert_eq!(r["complete"], true);
}

#[test]
fn dormant_capture_writes_are_not_claimed_and_each_call_reinitializes() {
    let r = finish(session(
        "program T; type Pair = record Read: function(): integer; Write: function(): integer; end record; function Make(): Pair; function Read(): integer; begin return Total; end function; function Write(): integer; begin Total := Total + 1; return Total; end function; begin var Total := 0; return Pair(Read := Read, Write := Write); end function; begin const P := Make(); discard P.Read(); discard P.Write(); discard P.Write(); discard P.Read(); end.",
        &["Make.Read::Total"],
    ));
    assert_eq!(values(&r, 0), vec![json!("0"), json!("2")]);
    assert!(
        r["events"]
            .as_array()
            .unwrap()
            .iter()
            .all(|e| e["kind"] == "initialize")
    );
    assert_eq!(r["complete"], true);
}

#[test]
fn repeated_admissions_and_debug_tail_call_frames_renew_activations() {
    let source = "program T; function Make(Base: integer): function(N: integer): integer; function Next(N: integer): integer; begin if N > 0 then return Next(N - 1); end if; return Base; end function; begin return Next; end function; begin const A := Make(7); var I := 0; while I < 1100 do discard A(0); I := I + 1; end while; discard A(3); end.";
    let (p, errors) = fpas_parser::parse(source);
    assert!(errors.is_empty());
    let image = fpas_compiler::compile(&p).unwrap();
    assert!(
        image
            .executable()
            .code
            .iter()
            .any(|word| word.opcode() == Ok(Opcode::TailCallValue))
    );
    let mut s = DebugSession::new(image).unwrap();
    s.install_trace(&["Make.Next::Base".into()], 2000).unwrap();
    let r = finish(s);
    assert_eq!(r["event_count"], 1104);
    let ids: std::collections::BTreeSet<_> = r["events"]
        .as_array()
        .unwrap()
        .iter()
        .map(|e| e["activation_id"].as_str().unwrap())
        .collect();
    assert_eq!(ids.len(), 1104);
    assert_eq!(r["complete"], true);
}

#[test]
fn inline_callback_debug_frames_keep_each_recursive_capture_activation_live() {
    let r = finish(session(
        "program T; function Make(): function(N: integer): integer; function Next(N: integer): integer; begin Total := Total + 1; if N > 0 then return Next(N - 1); end if; return Total; end function; begin var Total := 0; return Next; end function; begin const A := Make(); discard [3].Map(A); end.",
        &["Make.Next::Total"],
    ));
    assert_eq!(
        values(&r, 0),
        vec![
            json!("0"),
            json!("1"),
            json!("1"),
            json!("2"),
            json!("2"),
            json!("2"),
            json!("3"),
            json!("3"),
            json!("3"),
            json!("3"),
            json!("4"),
            json!("4"),
            json!("4"),
            json!("4")
        ]
    );
    let events = r["events"].as_array().unwrap();
    let mut activations = std::collections::BTreeMap::<_, Vec<_>>::new();
    for event in events {
        activations
            .entry(event["activation_id"].as_str().unwrap())
            .or_default()
            .push(event);
    }
    for activation in activations.values() {
        assert_eq!(activation[0]["kind"], "initialize");
        for event in &activation[1..] {
            assert_eq!(event["kind"], "change");
            assert_eq!(event["instance_id"], activation[0]["instance_id"]);
        }
    }
    assert_eq!(
        activations
            .values()
            .map(Vec::len)
            .collect::<std::collections::BTreeSet<_>>(),
        [2, 3, 4, 5].into()
    );
    let ids: std::collections::BTreeSet<_> = events
        .iter()
        .map(|e| e["activation_id"].as_str().unwrap())
        .collect();
    assert_eq!(ids.len(), 4);
    assert_eq!(r["complete"], true);
}

#[test]
fn capture_and_owner_aliases_are_ordered_at_the_actual_writer_commit() {
    let r = finish(session(
        "program T; var Global: integer := 0; procedure Outer(var Total: integer); procedure Next(); begin Total := Total + 1; end procedure; begin Next(); end procedure; begin Outer(var Global); end.",
        &["Outer.Next::Total", "Global", "Outer::Total"],
    ));
    assert_eq!(values(&r, 0), vec![json!("0"), json!("1")]);
    let changes: Vec<_> = r["events"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|e| e["kind"] == "change")
        .collect();
    assert_eq!(
        changes
            .iter()
            .map(|e| e["watch_id"].clone())
            .collect::<Vec<_>>(),
        json!([0, 1, 2]).as_array().unwrap().clone()
    );
    assert!(changes.iter().all(|e| e["function"] == "outer.next"));
    assert_ne!(changes[0]["instance_id"], changes[2]["instance_id"]);
    assert_eq!(r["complete"], true);
}

#[test]
fn worker_callbacks_reuse_activations_without_leaking_capture_leases() {
    let r = finish(session(
        "program T; uses Std.Tasks as Tasks; function Work(): integer; function Add(N: integer): integer; begin Total := Total + N; return Total; end function; begin var Total := 0; discard [1, 2, 3].Map(Add); return Total; end function; begin scope const A := go Work(); const B := go Work(); discard Tasks.Wait(A); discard Tasks.Wait(B); end scope; end.",
        &["Work.Add::Total", "Work::Total"],
    ));
    for task in [1, 2] {
        let events: Vec<_> = r["events"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|e| e["watch_id"] == 0 && e["task_id"] == task)
            .collect();
        assert_eq!(
            events
                .iter()
                .map(|e| e["new_value"]["scalar"].clone())
                .collect::<Vec<_>>(),
            vec![
                json!("0"),
                json!("1"),
                json!("1"),
                json!("3"),
                json!("3"),
                json!("6")
            ]
        );
        let ids: std::collections::BTreeSet<_> = events
            .iter()
            .map(|e| e["instance_id"].as_str().unwrap())
            .collect();
        assert_eq!(ids.len(), 3);
    }
    assert_eq!(r["complete"], true);
}

#[test]
fn failed_capture_store_keeps_history_and_caps_have_exact_prefix_counts() {
    let source = "program T; function Make(): function(): integer; function Fail(): integer; begin Total := 2; Total := 1 div 0; return Total; end function; begin var Total := 1; return Fail; end function; begin const A := Make(); discard A(); end.";
    let mut s = session(source, &["Make.Fail::Total"]);
    assert_eq!(
        stopped(s.continue_execution().unwrap()).reason,
        DebugStopReason::RuntimeError
    );
    let r = serde_json::to_value(s.take_trace().unwrap()).unwrap();
    assert_eq!(values(&r, 0), vec![json!("1"), json!("2")]);
    assert_eq!(r["complete"], true);
    let mut s = session(source, &["Make.Fail::Total"]);
    s.install_trace(&["Make.Fail::Total".into()], 1).unwrap();
    assert_eq!(
        s.continue_execution().unwrap_err().kind,
        DebugErrorKind::TraceEventLimit
    );
    let r = serde_json::to_value(s.take_trace().unwrap()).unwrap();
    assert_eq!(r["total_count"], 2);
    assert_eq!(values(&r, 0), vec![json!("1")]);
    assert_eq!(r["complete"], false);
}

#[test]
fn capture_live_and_string_limits_preserve_unknown_coverage() {
    for (ty, value, calls) in [
        ("integer", "7", 1100),
        ("string", "'x'.RepeatStr(65536)", 200),
    ] {
        let source = format!(
            "program T; function Make(Base: {ty}): function(N: integer): integer; function Next(N: integer): integer; begin if N > 0 then const R := Next(N - 1); return R + 1; end if; discard Base; return 0; end function; begin return Next; end function; begin const A := Make({value}); discard A({calls}); end."
        );
        let mut s = session(&source, &["Make.Next::Base"]);
        s.install_trace(&["Make.Next::Base".into()], 10000).unwrap();
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
fn inconsistent_or_cyclic_external_capture_provenance_rejects_before_execution() {
    use crate::{DebugBindingId, DebugBindingKind, DebugCaptureKind, FunctionId};
    let (p, errors) = fpas_parser::parse(
        "program T; function Make(): function(): integer; function Next(): integer; begin Total := Total + 1; return Total; end function; begin var Total := 0; return Next; end function; begin const A := Make(); discard A(); end.",
    );
    assert!(errors.is_empty());
    let mut original = fpas_compiler::compile(&p).unwrap().into_unverified();
    for function in &mut original.functions {
        for binding in &mut function.debug.bindings {
            binding.projection = None;
        }
    }
    let f = original
        .functions
        .iter()
        .position(|f| original.strings.get(f.name) == Some("make.next"))
        .unwrap();
    let b = original.functions[f]
        .debug
        .bindings
        .iter()
        .position(|b| b.kind == DebugBindingKind::Capture)
        .unwrap();
    for case in 0..6 {
        let mut image = original.clone();
        match case {
            0 => {
                let mut extra = image.functions[f].debug.bindings[b].clone();
                extra.hidden = true;
                image.functions[f].debug.bindings.push(extra);
            }
            1 => {
                let boolean = image.debug_types.len();
                image.debug_types.push(crate::DebugType::Boolean);
                image.functions[f].debug.bindings[b].ty = crate::DebugTypeId::new(boolean as u32);
            }
            2 => {
                image.functions[f].debug.lexical_owner = Some(FunctionId::new(f as u16));
                image.functions[f].debug.capture_sources[0].binding = DebugBindingId::new(b as u32);
                image.functions[f].debug.capture_sources[0].kind = DebugCaptureKind::EnclosingCell;
            }
            3 => {
                let owner = image.functions[f].debug.lexical_owner.unwrap().get() as usize;
                let source = image.functions[f].debug.capture_sources[0].binding.get() as usize;
                image.functions[owner].debug.bindings[source].hidden = true;
            }
            4 => {
                image.functions[f].debug.bindings[b].declaration = None;
            }
            _ => {
                let owner = image.functions[f].debug.lexical_owner.unwrap().get() as usize;
                let source = image.functions[f].debug.capture_sources[0].binding.get() as usize;
                image.functions[owner].debug.bindings[source].name =
                    image.functions[f].debug.bindings[b].type_name;
            }
        }
        let mut s = DebugSession::new(image.verify().unwrap()).unwrap();
        assert!(
            s.install_trace(&["Make.Next::Total".into()], 100).is_err(),
            "case {case}"
        );
        assert!(s.take_trace().is_none());
    }
}

#[test]
fn wrong_runtime_capture_representation_ends_coverage_instead_of_following_an_alias() {
    use crate::{DebugBindingKind, DebugCaptureKind};
    let (p, errors) = fpas_parser::parse(
        "program T; function Make(): function(): integer; function Next(): integer; begin Total := Total + 1; return Total; end function; begin var Total := 0; return Next; end function; begin const A := Make(); discard A(); end.",
    );
    assert!(errors.is_empty());
    let mut image = fpas_compiler::compile(&p).unwrap().into_unverified();
    let f = image
        .functions
        .iter()
        .position(|f| image.strings.get(f.name) == Some("make.next"))
        .unwrap();
    let source = image.functions[f].debug.capture_sources[0];
    let owner = image.functions[f].debug.lexical_owner.unwrap().get() as usize;
    image.functions[owner].debug.bindings[source.binding.get() as usize].cell_backed = false;
    let capture = image.functions[f]
        .debug
        .bindings
        .iter_mut()
        .find(|b| b.kind == DebugBindingKind::Capture)
        .unwrap();
    capture.cell_backed = false;
    image.functions[f].debug.capture_sources[0].kind = DebugCaptureKind::Value;
    let mut s = DebugSession::new(image.verify().unwrap()).unwrap();
    s.install_trace(&["Make.Next::Total".into()], 10).unwrap();
    assert_eq!(
        s.continue_execution().unwrap_err().kind,
        DebugErrorKind::TraceValueLimit
    );
    let r = serde_json::to_value(s.take_trace().unwrap()).unwrap();
    assert_eq!(r["complete"], false);
    assert!(r["total_count"].is_null());
    assert_eq!(r["event_count"], 0);
}
