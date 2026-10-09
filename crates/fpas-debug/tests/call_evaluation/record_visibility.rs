//! Unit-scoped typed construction after object linking.
//! See `docs/pascal/language/types/records.md`.

use super::*;

const UNIT: &str = "unit Geometry;
public type Point = record public X: integer := 5; public Y: integer := 6; end record;
public type Secure = record Value: integer := 7; end record;
public procedure PauseHere();
begin
  const UnitMarker: integer := 0;
end procedure;
end unit;";

fn linked(source: &str) -> fpas_bytecode::VerifiedExecutable {
    let (unit, diagnostics) = fpas_parser::parse_compilation_unit(UNIT);
    assert!(diagnostics.is_empty(), "{diagnostics:?}");
    let fpas_parser::CompilationUnit::Unit(unit) = unit else {
        panic!("unit");
    };
    let mut unit = fpas_compiler::compile_unit_object(&unit, &[]).expect("unit object");
    unit.object.sources = vec!["geometry.fpas".into()];
    let (program, diagnostics) = fpas_parser::parse(source);
    assert!(diagnostics.is_empty(), "{diagnostics:?}");
    let interfaces = [unit.interface];
    let mut root =
        fpas_compiler::compile_program_object_with_support(&program, &interfaces, &interfaces)
            .expect("root object");
    root.sources = vec!["consumer.fpas".into()];
    let unit = fpas_unit::object::decode_object(
        &fpas_unit::object::encode_object(&unit.object).expect("encode unit"),
    )
    .expect("decode unit");
    fpas_linker::link_objects(&[unit], &root).expect("link constructors")
}

fn record(name: &str, fields: Vec<(String, DebugExpression)>) -> DebugExpression {
    DebugExpression::Record {
        name: name.into(),
        fields,
    }
}

#[test]
fn uses_aliases_keep_nominal_types_defaults_and_private_construction_rules() {
    let mut session = DebugSession::new(linked(
        "program Consumer; uses Geometry as G; type LocalPoint = G.Point; begin end.",
    ))
    .expect("session");
    let value = session
        .evaluate(
            &record("G.Point", vec![("Y".into(), DebugExpression::Integer(2))]),
            None,
        )
        .expect("uses alias constructor");
    assert_eq!(value.type_name.to_ascii_lowercase(), "geometry.point");
    let field = DebugExpression::Field {
        base: Box::new(record("G.Point", vec![])),
        name: "X".into(),
    };
    assert_eq!(
        session
            .evaluate(&field, None)
            .expect("imported default")
            .value,
        "5"
    );
    let failure = session
        .evaluate(&record("G.Secure", vec![]), None)
        .expect_err("private field constructor");
    assert_eq!(failure.kind, DebugErrorKind::EvaluationType, "{failure:?}");
    assert!(failure.message.contains("declaring unit"));
    assert_eq!(
        session
            .evaluate(
                &DebugExpression::Field {
                    base: Box::new(record("LocalPoint", vec![])),
                    name: "X".into(),
                },
                None
            )
            .expect("source alias of imported type")
            .value,
        "5"
    );
    for hidden in ["Point", "Geometry.Point"] {
        assert_eq!(
            session
                .evaluate(&record(hidden, vec![]), None)
                .expect_err("uses alias namespace")
                .kind,
            DebugErrorKind::UnknownCallable
        );
    }
}

#[test]
fn available_unit_types_require_a_source_import() {
    let mut session = DebugSession::new(linked("program Consumer; begin end.")).expect("session");
    for name in ["Point", "Geometry.Point"] {
        assert_eq!(
            session
                .evaluate(&record(name, vec![]), None)
                .expect_err("missing uses")
                .kind,
            DebugErrorKind::UnknownCallable
        );
    }
}

#[test]
fn private_records_are_constructible_in_the_declaring_unit_frame() {
    let mut session = DebugSession::new(linked(
        "program Consumer; uses Geometry as G; begin G.PauseHere(); end.",
    ))
    .expect("session");
    let breakpoint = session
        .set_breakpoint(SourceBreakpoint {
            source: "geometry.fpas".into(),
            line: 6,
            column: None,
        })
        .expect("unit breakpoint");
    assert!(breakpoint.is_verified(), "{breakpoint:?}");
    assert!(matches!(
        session.continue_execution().expect("unit stop"),
        DebugRunResult::Stopped(_)
    ));
    let frame = session.stack(0, 1).expect("stack").items[0].id;
    let field = DebugExpression::Field {
        base: Box::new(record("Secure", vec![])),
        name: "Value".into(),
    };
    assert_eq!(
        session
            .evaluate(&field, Some(frame))
            .expect("private owner construction")
            .value,
        "7"
    );
    let failure = session
        .evaluate(&record("G.Point", vec![]), Some(frame))
        .expect_err("consumer alias is not a unit alias");
    assert_eq!(failure.kind, DebugErrorKind::UnknownCallable);
}
