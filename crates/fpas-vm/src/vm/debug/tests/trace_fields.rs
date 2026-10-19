//! Scalar field projection, shared declaration lifetimes and source-store coverage.

use super::trace_locals::{finish, session, values};
use super::*;
use serde_json::json;

#[test]
fn nested_fields_ignore_equal_replacements_and_preserve_nominal_leaf_types() {
    let r = finish(session(
        "program T; type Shade = enum Red; Blue; end enum; type Cursor = record Row: integer; Other: integer; end record; type Model = record At: Cursor; Color: Shade; end record; function F(Input: Model): integer; begin var State := Input; State.At.Other := 9; State.At.Row := 2; State := Model(At := Cursor(Row := 2, Other := 44), Color := Shade.Blue); State.At.Row := 1; return State.At.Row; end function; begin discard F(Model(At := Cursor(Row := 1, Other := 0), Color := Shade.Red)); end.",
        &[
            "F::State.At.Row",
            "F::Input.At.Row",
            "F::State.Color",
            "f::state.at.row",
        ],
    ));
    assert_eq!(values(&r, 0), vec![json!("1"), json!("2"), json!("1")]);
    assert_eq!(values(&r, 1), vec![json!("1")]);
    assert_eq!(values(&r, 2), vec![json!("0"), json!("1")]);
    assert_eq!(r["watches"].as_array().unwrap().len(), 3);
    assert_eq!(r["watches"][2]["type"], "Shade");
    assert_eq!(r["watches"][0]["identity"]["field_slots"], json!([0, 0]));
    let initialized: Vec<_> = r["events"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|e| e["kind"] == "initialize" && e["watch_id"] != 1)
        .collect();
    assert_eq!(initialized[0]["instance_id"], initialized[1]["instance_id"]);
    assert_eq!(r["complete"], true);
}

#[test]
fn record_and_scalar_var_aliases_preserve_each_leaf_and_request_order() {
    let r = finish(session(
        "program T; type Model = record X: integer; Y: integer; end record; procedure SetValue(var N: integer); begin N := 3; end procedure; procedure Change(var M: Model); begin M.X := 2; SetValue(var M.X); M := Model(X := 1, Y := 9); end procedure; procedure Caller(); begin var State := Model(X := 1, Y := 0); Change(var State); end procedure; begin Caller(); end.",
        &[
            "Change::M.X",
            "Caller::State.X",
            "Caller::State.Y",
            "SetValue::N",
        ],
    ));
    assert_eq!(
        values(&r, 0),
        vec![json!("1"), json!("2"), json!("3"), json!("1")]
    );
    assert_eq!(values(&r, 1), values(&r, 0));
    assert_eq!(values(&r, 2), vec![json!("0"), json!("9")]);
    assert_eq!(values(&r, 3), vec![json!("2"), json!("3")]);
    let changes: Vec<_> = r["events"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|e| e["kind"] == "change")
        .map(|e| e["watch_id"].as_u64().unwrap())
        .collect();
    assert_eq!(changes, vec![0, 1, 0, 1, 3, 0, 1, 2]);
}

#[test]
fn global_and_local_field_aliases_share_the_actual_commit() {
    let r = finish(session(
        "program T; type Model = record X: integer; end record; var State: Model := Model(X := 0); procedure Change(var M: Model); begin M.X := 1; M := Model(X := 2); end procedure; begin Change(var State); end.",
        &["Change::M.X", "State.X"],
    ));
    let changes: Vec<_> = r["events"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|e| e["kind"] == "change")
        .map(|e| e["watch_id"].as_u64().unwrap())
        .collect();
    assert_eq!(changes, vec![0, 1, 0, 1]);
}

#[test]
fn nested_cached_callbacks_follow_cell_backed_record_replacement() {
    let r = finish(session(
        "program T; type Model = record X: integer; Y: integer; end record; procedure Outer(); begin var State := Model(X := 0, Y := 0); discard [1, 2, 1].Map(function(N: integer): integer begin discard [N].Map(function(M: integer): integer begin State := Model(X := M, Y := M + 1); return M; end function); return N; end function); end procedure; begin Outer(); end.",
        &["Outer::State.Y", "Outer::State.X"],
    ));
    assert_eq!(
        values(&r, 0),
        vec![json!("0"), json!("2"), json!("3"), json!("2")]
    );
    assert_eq!(
        values(&r, 1),
        vec![json!("0"), json!("1"), json!("2"), json!("1")]
    );
    for event in r["events"].as_array().unwrap().iter().skip(2) {
        assert_ne!(event["function"], "outer");
    }
}

#[test]
fn repeated_declarations_share_ids_per_root_and_retire_every_selected_path() {
    let mut s = session(
        "program T; type Model = record X: integer; Y: integer; end record; procedure Loop(); begin for I: integer := 1 to 1100 do const State := Model(X := I, Y := I); end for; end procedure; begin Loop(); end.",
        &["Loop::State.X", "Loop::State.Y"],
    );
    s.install_trace(&["Loop::State.X".into(), "Loop::State.Y".into()], 10000)
        .unwrap();
    let r = finish(s);
    assert_eq!(r["event_count"], 2200);
    let events = r["events"].as_array().unwrap();
    for pair in events.as_chunks::<2>().0 {
        assert_eq!(pair[0]["instance_id"], pair[1]["instance_id"]);
    }
    assert_ne!(events[0]["instance_id"], events[2]["instance_id"]);
}

#[test]
fn worker_callbacks_keep_field_scope_and_delayed_source_locations() {
    let source = "program T;\nuses Std.Tasks as Tasks;\ntype Model = record X: integer; end record;\nfunction Add(A: integer; B: integer): integer; begin return A + B; end function;\nfunction Work(): integer;\nbegin\n begin\n var State := Model(X := 0);\n State.X := [1, 2].Reduce(0, Add);\n discard [1, 2, 3].Map(function(N: integer): integer begin State.X := N; return N; end function);\n end;\n return 0;\nend function;\nbegin scope const Job := go Work(); discard Tasks.Wait(Job); end scope; end.";
    let r = finish(session(source, &["Work::State.X"]));
    assert_eq!(
        values(&r, 0),
        vec![json!("0"), json!("3"), json!("1"), json!("2"), json!("3")]
    );
    assert_eq!(r["events"][1]["location"]["line"], 9);
    for event in r["events"].as_array().unwrap().iter().skip(2) {
        assert_eq!(event["task_id"], 1);
        assert_eq!(event["location"]["line"], 10);
    }
}

#[test]
fn live_path_limit_counts_multiple_leaves_even_with_a_shared_declaration_id() {
    let mut s = session(
        "program T; type Model = record X: integer; Y: integer; end record; function F(N: integer; Input: Model): integer; begin if N = 0 then return 0; end if; const Next := F(N - 1, Input); return Next + 1; end function; begin discard F(600, Model(X := 1, Y := 2)); end.",
        &["F::Input.X", "F::Input.Y"],
    );
    s.install_trace(&["F::Input.X".into(), "F::Input.Y".into()], 10000)
        .unwrap();
    assert_eq!(
        s.continue_execution().unwrap_err().kind,
        DebugErrorKind::TraceValueLimit
    );
    let r = serde_json::to_value(s.take_trace().unwrap()).unwrap();
    assert_eq!(r["complete"], false);
    assert!(r["total_count"].is_null());
}

#[test]
fn invalid_field_selectors_never_install_a_partial_request() {
    let (p, errors) = fpas_parser::parse(
        "program T; type Model = record X: integer; end record; procedure F(); begin var State := Model(X := 0); end procedure; begin F(); end.",
    );
    assert!(errors.is_empty());
    let image = fpas_compiler::compile(&p).unwrap();
    for target in [
        "F::State",
        "F::State.Missing",
        "F::State.X.More",
        "F::State.X()",
        "F::State[0].X",
        "F::State..X",
    ] {
        let mut s = DebugSession::new(image.clone()).unwrap();
        assert!(
            s.install_trace(&["F::State.X".into(), target.into()], 10)
                .is_err(),
            "{target}"
        );
        assert!(s.take_trace().is_none());
    }
}

#[test]
fn applied_generic_fields_and_concrete_siblings_have_exact_histories() {
    let source = "program T; type Box of T = record Item: T; Other: integer; end record; procedure F(); begin var State: Box of integer := Box(Item := 1, Other := 0); State.Item := 2; State.Other := 3; end procedure; begin F(); end.";
    let r = finish(session(source, &["F::State.Other", "F::State.Item"]));
    assert_eq!(values(&r, 0), vec![json!("0"), json!("3")]);
    assert_eq!(values(&r, 1), vec![json!("1"), json!("2")]);
    assert_eq!(r["complete"], true);
}

#[test]
fn nominal_enum_layout_mismatch_is_unknown_coverage_instead_of_a_false_typed_event() {
    let (p, errors) = fpas_parser::parse(
        "program T; type A = enum First; Second(N: integer); end enum; type B = enum Third; Fourth(N: integer); end enum; type Model = record X: A; end record; var Other: B := B.Third; procedure F(); begin var State := Model(X := A.First); end procedure; begin F(); end.",
    );
    assert!(errors.is_empty());
    let mut image = fpas_compiler::compile(&p).unwrap().into_unverified();
    let b = image
        .enums
        .iter()
        .position(|e| image.strings.get(e.name) == Some("B"))
        .unwrap();
    let ty = image
        .debug_types
        .iter()
        .position(
            |ty| matches!(ty, fpas_bytecode::DebugType::Enum(id) if usize::from(id.get()) == b),
        )
        .unwrap();
    for variant in &mut image.enum_variants {
        if usize::from(variant.owner.get()) == b {
            variant.fields.clear();
            variant.field_types.clear();
        }
    }
    image.records[0].fields[0].ty = fpas_bytecode::DebugTypeId::new(ty as u32);
    image.records[0].fields[0].type_name = Some(image.enums[b].name);
    for function in &mut image.functions {
        for binding in &mut function.debug.bindings {
            binding.projection = None;
        }
    }
    let mut s = DebugSession::new(image.verify().unwrap()).unwrap();
    s.install_trace(&["F::State.X".into()], 100).unwrap();
    assert_eq!(
        s.continue_execution().unwrap_err().kind,
        DebugErrorKind::TraceValueLimit
    );
    let r = serde_json::to_value(s.take_trace().unwrap()).unwrap();
    assert_eq!(r["complete"], false);
    assert!(r["total_count"].is_null());
    assert_eq!(r["event_count"], 0);
}

#[test]
fn field_depth_limit_is_exact_and_unexpected_runtime_layouts_are_explicit() {
    for nested in [15, 16] {
        let mut source = "program T; type R0 = record X: integer; end record;".to_owned();
        let mut value = "R0(X := 1)".to_owned();
        for depth in 1..=nested {
            source.push_str(&format!(
                "type R{depth} = record Next: R{}; end record;",
                depth - 1
            ));
            value = format!("R{depth}(Next := {value})");
        }
        source.push_str(&format!(
            "procedure F(Input: R{nested}); begin null; end procedure; begin F({value}); end."
        ));
        let target = format!("F::Input{}", ".Next".repeat(nested)) + ".X";
        let (p, errors) = fpas_parser::parse(&source);
        assert!(errors.is_empty());
        let mut s = DebugSession::new(fpas_compiler::compile(&p).unwrap()).unwrap();
        if nested == 15 {
            s.install_trace(&[target], 10).unwrap();
            assert_eq!(values(&finish(s), 0), vec![json!("1")]);
        } else {
            assert!(s.install_trace(&[target], 10).is_err());
        }
    }
    let (p, errors) = fpas_parser::parse(
        "program T; type A = record X: integer; end record; type B = record X: integer; end record; var Other: B := B(X := 9); procedure F(); begin var State := A(X := 1); end procedure; begin F(); end.",
    );
    assert!(errors.is_empty());
    let mut image = fpas_compiler::compile(&p).unwrap().into_unverified();
    let b = image
        .records
        .iter()
        .position(|r| image.strings.get(r.name) == Some("B"))
        .unwrap();
    let ty = image
        .debug_types
        .iter()
        .position(
            |ty| matches!(ty, fpas_bytecode::DebugType::Record(id) if usize::from(id.get()) == b),
        )
        .unwrap();
    let function = image
        .functions
        .iter_mut()
        .find(|f| image.strings.get(f.name) == Some("f"))
        .unwrap();
    let binding = function
        .debug
        .bindings
        .iter_mut()
        .find(|b| image.strings.get(b.name) == Some("State"))
        .unwrap();
    binding.ty = fpas_bytecode::DebugTypeId::new(ty as u32);
    binding.projection = None;
    let mut s = DebugSession::new(image.verify().unwrap()).unwrap();
    s.install_trace(&["F::State.X".into()], 10).unwrap();
    assert_eq!(
        s.continue_execution().unwrap_err().kind,
        DebugErrorKind::TraceValueLimit
    );
    let r = serde_json::to_value(s.take_trace().unwrap()).unwrap();
    assert_eq!(r["complete"], false);
    assert!(r["total_count"].is_null());
    assert_eq!(r["event_count"], 0);
}
