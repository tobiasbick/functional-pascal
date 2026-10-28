//! Concrete source projections remain separate from erased runtime storage.

use super::trace_locals::{finish, values};
use super::*;
use crate::{DebugProjection, DebugProjectionKind, DebugType, DebugTypeId};
use serde_json::json;

fn image(source: &str) -> crate::Executable {
    let (program, errors) = fpas_parser::parse(source);
    assert!(errors.is_empty(), "{errors:?}");
    fpas_compiler::compile(&program).unwrap().into_unverified()
}

fn scalar(image: &crate::Executable, expected: DebugType) -> DebugProjection {
    let ty = image
        .debug_types
        .iter()
        .position(|ty| *ty == expected)
        .unwrap();
    DebugProjection {
        ty: DebugTypeId::new(ty as u32),
        type_name: None,
        kind: DebugProjectionKind::Scalar,
    }
}

#[test]
fn distinct_applied_roots_sharing_one_layout_keep_separate_concrete_paths() {
    let mut image = image(
        "program T; type Box of T = record Item: T; Other: integer; end record; var Numbers: Box of integer := Box(Item := 1, Other := 0); var Words: Box of string := Box(Item := 'first', Other := 0); begin Numbers.Item := 2; Words.Item := 'last'; end.",
    );
    assert_eq!(image.globals[0].ty, image.globals[1].ty);
    let integer = scalar(&image, DebugType::Integer);
    let string = scalar(&image, DebugType::String);
    for (root, item) in [integer.clone(), string].into_iter().enumerate() {
        image.globals[root].projection = Some(DebugProjection {
            ty: image.globals[root].ty,
            type_name: None,
            kind: DebugProjectionKind::Record(vec![item, integer.clone()]),
        });
    }
    let mut session = DebugSession::new(image.verify().unwrap()).unwrap();
    session
        .install_trace(&["Numbers.Item".into(), "Words.Item".into()], 10)
        .unwrap();
    let report = finish(session);
    assert_eq!(values(&report, 0), vec![json!("1"), json!("2")]);
    assert_eq!(values(&report, 1), vec![json!("first"), json!("last")]);
    assert_eq!(report["watches"][0]["type"], "integer");
    assert_eq!(report["watches"][1]["type"], "string");
    assert_eq!(report["complete"], true);
}

#[test]
fn absent_projection_metadata_preserves_legacy_concrete_fields_without_guessing() {
    let mut image = image(
        "program T; type Box of T = record Item: T; Other: integer; end record; procedure F(); begin var State: Box of integer := Box(Item := 1, Other := 0); State.Other := 2; end procedure; begin F(); end.",
    );
    for function in &mut image.functions {
        for binding in &mut function.debug.bindings {
            binding.projection = None;
        }
    }
    let verified = image.verify().unwrap();
    let mut session = DebugSession::new(verified.clone()).unwrap();
    session
        .install_trace(&["F::State.Other".into()], 10)
        .unwrap();
    assert_eq!(values(&finish(session), 0), vec![json!("0"), json!("2")]);
    let mut session = DebugSession::new(verified).unwrap();
    assert!(
        session
            .install_trace(&["F::State.Other".into(), "F::State.Item".into()], 10)
            .is_err()
    );
    assert!(session.take_trace().is_none());
}

#[test]
fn unsupported_concrete_child_rejects_the_entire_request_before_execution() {
    let mut image = image(
        "program T; type Box of T = record Item: T; Other: integer; end record; var State: Box of integer := Box(Item := 1, Other := 0); begin State.Other := 2; end.",
    );
    let integer = scalar(&image, DebugType::Integer);
    let mut unsupported = integer.clone();
    unsupported.kind = DebugProjectionKind::Unsupported;
    image.globals[0].projection = Some(DebugProjection {
        ty: image.globals[0].ty,
        type_name: None,
        kind: DebugProjectionKind::Record(vec![unsupported, integer]),
    });
    let mut session = DebugSession::new(image.verify().unwrap()).unwrap();
    assert!(
        session
            .install_trace(&["State.Other".into(), "State.Item".into()], 10)
            .is_err()
    );
    assert!(session.take_trace().is_none());
}

#[test]
fn operational_cutoff_preserves_known_subtrees_without_refining_erased_fields() {
    let mut image = image(
        "program T; type Box of T = record Item: T; Other: integer; end record; type Model = record Boxes: array of Box of integer; end record; var State: Model := Model(Boxes := [Box(Item := 1, Other := 0)]); begin State.Boxes[0].Other := 2; end.",
    );
    let DebugType::Record(record) = image.debug_types[image.globals[0].ty.get() as usize] else {
        panic!("expected record root");
    };
    let child_type = image.records[record.get() as usize].fields[0].ty;
    image.globals[0].projection = Some(DebugProjection {
        ty: image.globals[0].ty,
        type_name: None,
        kind: DebugProjectionKind::Record(vec![DebugProjection {
            ty: child_type,
            type_name: None,
            kind: DebugProjectionKind::Operational,
        }]),
    });
    let verified = image.verify().unwrap();
    let mut session = DebugSession::new(verified.clone()).unwrap();
    session
        .install_trace(&["State.Boxes[0].Other".into()], 10)
        .unwrap();
    assert_eq!(values(&finish(session), 0), vec![json!("0"), json!("2")]);
    let mut session = DebugSession::new(verified).unwrap();
    assert!(
        session
            .install_trace(
                &["State.Boxes[0].Other".into(), "State.Boxes[0].Item".into()],
                10,
            )
            .is_err()
    );
    assert!(session.take_trace().is_none());
}

#[test]
fn operational_reference_roots_preserve_existing_var_alias_traces() {
    let mut image = image(
        "program T; type Model = record Value: integer; end record; var State: Model := Model(Value := 1); procedure Change(var Input: Model); begin Input.Value := 2; end procedure; begin Change(var State); end.",
    );
    for global in &mut image.globals {
        global.projection = Some(DebugProjection {
            ty: global.ty,
            type_name: None,
            kind: DebugProjectionKind::Operational,
        });
    }
    for function in &mut image.functions {
        for binding in &mut function.debug.bindings {
            binding.projection = Some(DebugProjection {
                ty: binding.ty,
                type_name: None,
                kind: DebugProjectionKind::Operational,
            });
        }
    }
    let mut session = DebugSession::new(image.verify().unwrap()).unwrap();
    session
        .install_trace(&["Change::Input.Value".into(), "State.Value".into()], 10)
        .unwrap();
    let report = finish(session);
    assert_eq!(values(&report, 0), vec![json!("1"), json!("2")]);
    assert_eq!(values(&report, 1), vec![json!("1"), json!("2")]);
    assert_eq!(report["complete"], true);
}

#[test]
fn runtime_values_cannot_override_concrete_projection_types() {
    let mut image = image(
        "program T; type Box of T = record Item: T; Other: integer; end record; var State: Box of integer := Box(Item := 1, Other := 0); begin State.Item := 2; end.",
    );
    image.globals[0].projection = Some(DebugProjection {
        ty: image.globals[0].ty,
        type_name: None,
        kind: DebugProjectionKind::Record(vec![
            scalar(&image, DebugType::String),
            scalar(&image, DebugType::Integer),
        ]),
    });
    let mut session = DebugSession::new(image.verify().unwrap()).unwrap();
    session.install_trace(&["State.Item".into()], 10).unwrap();
    assert_eq!(
        session.continue_execution().unwrap_err().kind,
        DebugErrorKind::TraceValueLimit
    );
    let report = serde_json::to_value(session.take_trace().unwrap()).unwrap();
    assert_eq!(report["event_count"], 0);
    assert_eq!(report["complete"], false);
    assert!(report["total_count"].is_null());
}

#[test]
fn capture_projection_must_agree_with_its_exact_source_even_for_erased_layouts() {
    let mut image = image(
        "program T; type Box of T = record Item: T; end record; function Make(): function(): integer; function Read(): integer; begin return State.Item; end function; begin const State: Box of integer := Box(Item := 1); return Read; end function; begin const Read := Make(); discard Read(); end.",
    );
    let string = scalar(&image, DebugType::String);
    let function = image
        .functions
        .iter_mut()
        .find(|function| image.strings.get(function.name) == Some("make.read"))
        .unwrap();
    let capture = function
        .debug
        .bindings
        .iter_mut()
        .find(|binding| binding.kind == crate::DebugBindingKind::Capture)
        .unwrap();
    capture.projection = Some(DebugProjection {
        ty: capture.ty,
        type_name: None,
        kind: DebugProjectionKind::Record(vec![string]),
    });
    // A portable-image validator may reject earlier than trace installation.
    if let Ok(image) = image.verify() {
        let mut session = DebugSession::new(image).unwrap();
        assert!(
            session
                .install_trace(&["Make.Read::State.Item".into()], 10)
                .is_err()
        );
        assert!(session.take_trace().is_none());
    }
}
